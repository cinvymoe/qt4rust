// 存储管道（事件驱动版本）
//
// 职责: 只处理数据流（接收、缓冲、定时刷盘）。
// 业务逻辑（报警防抖、批量写入、数据清理）由 StorageService 处理。

use super::alarm_debouncer::AlarmAction;
use super::event_channel::{create_storage_channels, StorageEventReceiver};
use super::storage_service::StorageService;
use crate::models::ProcessedData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 存储管道配置 — 只包含数据流相关的配置
#[derive(Debug, Clone)]
pub struct StoragePipelineConfig {
    /// 存储间隔（运行数据）
    pub interval: Duration,

    /// 批量存储大小
    pub batch_size: usize,

    /// 管道队列最大容量
    pub max_queue_size: usize,

    /// 是否只保存最新数据
    pub save_only_latest: bool,
}

impl Default for StoragePipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            batch_size: 10,
            max_queue_size: 1000,
            save_only_latest: false,
        }
    }
}

impl StoragePipelineConfig {
    /// 从全局配置拆分为管道配置和服务配置
    pub fn from_pipeline_config(
        config: &crate::config::pipeline_config::StorageConfig,
    ) -> (Self, super::storage_service::StorageServiceConfig) {
        use super::alarm_debouncer::AlarmDebounceConfig;
        use super::storage_service::StorageServiceConfig;

        let pipeline_config = Self {
            interval: Duration::from_millis(config.interval_ms),
            batch_size: config.batch_size,
            max_queue_size: config.max_queue_size,
            save_only_latest: config.save_only_latest,
        };

        let service_config = StorageServiceConfig {
            max_records: config.max_records,
            purge_threshold: config.purge_threshold,
            alarm_max_records: config.alarm_max_records,
            alarm_purge_threshold: config.alarm_purge_threshold,
            max_retries: config.max_retries,
            retry_delay: Duration::from_millis(config.retry_delay_ms),
            alarm_debounce: AlarmDebounceConfig {
                alarm_debounce_count: 5,
                alarm_clear_debounce_count: 10,
            },
        };

        (pipeline_config, service_config)
    }
}

/// 存储管道（事件驱动架构）
pub struct StoragePipeline {
    config: StoragePipelineConfig,
    service: Arc<StorageService>,
    running: Arc<AtomicBool>,
    handle: Option<tokio::task::JoinHandle<()>>,

    // 事件驱动新字段
    event_receiver: StorageEventReceiver,
    pending_batch: Vec<ProcessedData>,
    last_stored_sequence: Arc<AtomicU64>,
}

impl StoragePipeline {
    /// Create storage pipeline with event receiver
    pub fn with_event_channel(
        config: StoragePipelineConfig,
        service: Arc<StorageService>,
        event_receiver: StorageEventReceiver,
    ) -> Self {
        let batch_size = config.batch_size;
        Self {
            config,
            service,
            event_receiver,
            pending_batch: Vec::with_capacity(batch_size),
            last_stored_sequence: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Legacy constructor (keep for backward compatibility)
    pub async fn new(
        config: StoragePipelineConfig,
        repository: std::sync::Arc<dyn crate::repositories::storage_repository::StorageRepository>,
        _buffer: super::shared_buffer::SharedBuffer,
    ) -> Result<Self, String> {
        let (_tx, rx) = create_storage_channels(config.max_queue_size);
        let service_config = super::storage_service::StorageServiceConfig::default();
        let service = Arc::new(StorageService::new(repository, service_config));
        let mut pipeline = Self::with_event_channel(config, service, rx);

        pipeline.initialize_sequence().await?;

        Ok(pipeline)
    }

    /// Start the storage pipeline (uses global runtime)
    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("Storage pipeline already running".to_string());
        }

        self.running.store(true, Ordering::Relaxed);

        let pipeline = self.clone_for_callback();

        // Use global runtime to spawn the async task
        self.handle = Some(
            qt_threading_utils::runtime::global_runtime().spawn(async move {
                pipeline.run_event_loop().await;
            }),
        );

        Ok(())
    }

    /// Initialize last_stored_sequence from database (call before start)
    pub async fn initialize_sequence(&mut self) -> Result<(), String> {
        self.service.initialize_sequence().await?;
        self.last_stored_sequence.store(
            self.service.last_stored_sequence(),
            Ordering::Relaxed,
        );
        tracing::info!(
            "Storage last_seq initialized to {}",
            self.last_stored_sequence.load(Ordering::Relaxed)
        );
        Ok(())
    }

    /// Event-driven main loop
    async fn run_event_loop(&self) {
        let mut flush_interval = tokio::time::interval(self.config.interval);
        let mut shutdown_rx = self.event_receiver.shutdown_rx.clone();
        let data_rx = self.event_receiver.data_rx.clone();
        let pending_batch = tokio::sync::Mutex::new(self.pending_batch.clone());
        let last_seq = self.last_stored_sequence.clone();
        let config = self.config.clone();
        let service = self.service.clone();

        loop {
            let mut data_rx_guard = data_rx.lock().await;
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("Storage pipeline shutdown signal received");
                        break;
                    }
                }

                data = data_rx_guard.recv() => {
                    drop(data_rx_guard);
                    if let Some(data_list) = data {
                        Self::handle_new_data(
                            data_list,
                            &pending_batch,
                            &last_seq,
                            &config,
                            &service,
                        ).await;
                    }
                }

                _ = flush_interval.tick() => {
                    drop(data_rx_guard);
                    Self::flush_pending_batch(
                        &pending_batch,
                        &last_seq,
                        &service
                    ).await;
                }
            }
        }

        let batch_len = pending_batch.lock().await.len();
        tracing::info!("Draining {} pending records before shutdown", batch_len);
        Self::flush_pending_batch(&pending_batch, &last_seq, &service).await;

        // 确保在事件循环结束时重置 running 标志
        self.running.store(false, Ordering::Relaxed);
        tracing::debug!("Storage pipeline event loop ended, running flag reset");
    }

    async fn handle_new_data(
        data_list: Vec<ProcessedData>,
        pending_batch: &tokio::sync::Mutex<Vec<ProcessedData>>,
        last_seq: &Arc<AtomicU64>,
        config: &StoragePipelineConfig,
        service: &Arc<StorageService>,
    ) {
        let last_seq_val = last_seq.load(Ordering::Acquire);

        for data in data_list {
            if data.sequence_number > last_seq_val {
                let alarm_action = service.process_alarm(&data);
                match alarm_action {
                    AlarmAction::TriggerAlarm(alarm_data) => {
                        service.save_alarm(alarm_data).await;
                    }
                    AlarmAction::ClearAlarm => {
                        tracing::debug!("Alarm cleared at sequence {}", data.sequence_number);
                    }
                    AlarmAction::None => {}
                }

                let mut batch = pending_batch.lock().await;
                batch.push(data);
            }
        }

        // Auto-flush if batch is full
        let mut batch = pending_batch.lock().await;
        if batch.len() >= config.batch_size {
            if config.save_only_latest {
                if batch.len() > 1 {
                    let drain_end = batch.len() - 1;
                    batch.drain(0..drain_end);
                }
            }
            drop(batch);
            Self::flush_pending_batch(pending_batch, last_seq, service).await;
        }
    }

    async fn flush_pending_batch(
        pending_batch: &tokio::sync::Mutex<Vec<ProcessedData>>,
        last_seq: &Arc<AtomicU64>,
        service: &Arc<StorageService>,
    ) {
        let data_to_save = {
            let mut batch = pending_batch.lock().await;
            if batch.is_empty() {
                return;
            }
            std::mem::take(&mut *batch)
        };

        let max_seq = data_to_save
            .iter()
            .map(|d| d.sequence_number)
            .max()
            .unwrap_or(0);

        let result = service.save_batch(&data_to_save).await;

        match result {
            Ok(_saved) => {
                last_seq.store(max_seq, Ordering::Release);
                service.purge_if_needed().await;
            }
            Err(e) => {
                tracing::error!("Failed to save batch: {:?}", e);
                let mut batch = pending_batch.lock().await;
                batch.extend(data_to_save);
            }
        }
    }

    /// Save alarm asynchronously (public API for callbacks)
    pub fn save_alarm_async(&self, data: ProcessedData) {
        let service = Arc::clone(&self.service);
        tokio::spawn(async move {
            service.save_alarm(data).await;
        });
    }

    /// Stop the storage pipeline (graceful)
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        // Send shutdown signal through event receiver
        self.event_receiver.shutdown();

        // Abort the handle but don't block
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    /// Get pending batch length
    pub fn queue_len(&self) -> usize {
        self.pending_batch.len()
    }

    /// Get last stored sequence
    pub fn last_stored_sequence(&self) -> u64 {
        self.last_stored_sequence.load(Ordering::Relaxed)
    }

    /// Clone for callback (matches old interface)
    pub fn clone_for_callback(&self) -> Self {
        Self {
            config: self.config.clone(),
            service: Arc::clone(&self.service),
            event_receiver: StorageEventReceiver {
                data_rx: self.event_receiver.data_rx.clone(),
                shutdown_rx: self.event_receiver.shutdown_rx.clone(),
            },
            pending_batch: Vec::with_capacity(self.config.batch_size),
            last_stored_sequence: Arc::clone(&self.last_stored_sequence),
            running: Arc::clone(&self.running),
            handle: None,
        }
    }

    /// Notify danger cleared (legacy compatibility)
    pub fn notify_danger_cleared(&self) {
        self.service.notify_danger_cleared();
        tracing::debug!("Danger state reset to false");
    }
}

impl Drop for StoragePipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::shared_buffer::ProcessedDataBuffer;
    use crate::pipeline::storage_service::StorageServiceConfig;
    use crate::repositories::mock_storage_repository::MockStorageRepository;
    use std::sync::RwLock;

    #[tokio::test]
    async fn test_new() {
        let repo = Arc::new(MockStorageRepository::new());
        let buffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));
        let config = StoragePipelineConfig::default();

        let pipeline = StoragePipeline::new(config, repo, buffer).await;

        assert!(pipeline.is_ok());
    }

    #[tokio::test]
    async fn test_with_event_channel() {
        let repo = Arc::new(MockStorageRepository::new());
        let service_config = StorageServiceConfig::default();
        let service = Arc::new(StorageService::new(repo, service_config));
        let config = StoragePipelineConfig::default();
        let (_tx, rx) = create_storage_channels(100);

        let pipeline = StoragePipeline::with_event_channel(config, service, rx);

        assert_eq!(pipeline.queue_len(), 0);
        assert_eq!(pipeline.last_stored_sequence(), 0);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let repo = Arc::new(MockStorageRepository::new());
        let service_config = StorageServiceConfig::default();
        let service = Arc::new(StorageService::new(repo, service_config));
        let config = StoragePipelineConfig::default();
        let (_tx, rx) = create_storage_channels(100);

        let mut pipeline = StoragePipeline::with_event_channel(config, service, rx);

        // Start should succeed
        assert!(pipeline.start().is_ok());

        // Starting again should fail
        assert!(pipeline.start().is_err());

        // Stop should not panic
        pipeline.stop();
    }

    #[tokio::test]
    async fn test_queue_operations() {
        let repo = Arc::new(MockStorageRepository::new());
        let service_config = StorageServiceConfig::default();
        let service = Arc::new(StorageService::new(repo, service_config));
        let config = StoragePipelineConfig::default();
        let (_tx, rx) = create_storage_channels(100);

        let pipeline = StoragePipeline::with_event_channel(config, service, rx);

        // Initial queue should be empty
        assert_eq!(pipeline.queue_len(), 0);
        assert_eq!(pipeline.last_stored_sequence(), 0);
    }
}
