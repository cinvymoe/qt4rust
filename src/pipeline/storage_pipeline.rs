// 存储管道（事件驱动版本）

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use super::event_channel::{StorageEventReceiver, create_storage_channels};
use super::retry_policy::{RetryConfig, with_retry};
use crate::repositories::storage_repository::StorageRepository;
use crate::models::ProcessedData;
use crate::pipeline::StorageError;

/// 存储管道配置
#[derive(Debug, Clone)]
pub struct StoragePipelineConfig {
    /// 存储间隔（运行数据）
    pub interval: Duration,
    
    /// 批量存储大小
    pub batch_size: usize,
    
    /// 失败重试次数
    pub max_retries: u32,
    
    /// 重试延迟
    pub retry_delay: Duration,
    
    /// 管道队列最大容量
    pub max_queue_size: usize,
    
    /// 数据库最大记录条数（0 表示不限制）
    pub max_records: usize,
    
    /// 清理阈值（0 表示使用默认值 max_records * 1.1）
    pub purge_threshold: usize,
    
    /// 报警记录最大条数（0 表示不限制）
    pub alarm_max_records: usize,
    
    pub alarm_purge_threshold: usize,

    pub save_only_latest: bool,
    
    /// 报警防抖计数阈值（连续多少次危险才触发报警，0 表示禁用防抖）
    pub alarm_debounce_count: u32,
    
    /// 报警解除防抖计数阈值（连续多少次安全才解除报警，0 表示禁用防抖）
    pub alarm_clear_debounce_count: u32,
}

impl Default for StoragePipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            batch_size: 10,
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            max_queue_size: 1000,
            max_records: 0,
            purge_threshold: 0,
            alarm_max_records: 0,
            alarm_purge_threshold: 0,
            save_only_latest: false,
            alarm_debounce_count: 5,        // 默认：连续 5 次危险才报警 (500ms)
            alarm_clear_debounce_count: 10, // 默认：连续 10 次安全才解除 (1s)
        }
    }
}

impl StoragePipelineConfig {
    pub fn from_pipeline_config(config: &crate::config::pipeline_config::StorageConfig) -> Self {
        Self {
            interval: Duration::from_millis(config.interval_ms),
            batch_size: config.batch_size,
            max_retries: config.max_retries,
            retry_delay: Duration::from_millis(config.retry_delay_ms),
            max_queue_size: config.max_queue_size,
            max_records: config.max_records,
            purge_threshold: config.purge_threshold,
            alarm_max_records: config.alarm_max_records,
            alarm_purge_threshold: config.alarm_purge_threshold,
            save_only_latest: config.save_only_latest,
            alarm_debounce_count: 5,        // 默认值
            alarm_clear_debounce_count: 10, // 默认值
        }
    }
}

/// 存储管道（事件驱动架构）
pub struct StoragePipeline {
    config: StoragePipelineConfig,
    repository: Arc<dyn StorageRepository>,
    running: Arc<AtomicBool>,
    handle: Option<tokio::task::JoinHandle<()>>,
    /// 上次报警状态（用于检测持续报警）
    last_was_danger: Arc<AtomicBool>,
    
    // 事件驱动新字段
    event_receiver: StorageEventReceiver,
    pending_batch: Vec<ProcessedData>,
    last_stored_sequence: Arc<AtomicU64>,
    
    // 报警防抖计数器
    danger_count: Arc<std::sync::atomic::AtomicU32>,
    safe_count: Arc<std::sync::atomic::AtomicU32>,
}

impl StoragePipeline {
    /// Create storage pipeline with event receiver
    pub fn with_event_channel(
        config: StoragePipelineConfig,
        repository: Arc<dyn StorageRepository>,
        event_receiver: StorageEventReceiver,
    ) -> Self {
        let batch_size = config.batch_size;
        Self {
            config,
            repository,
            event_receiver,
            pending_batch: Vec::with_capacity(batch_size),
            last_stored_sequence: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            last_was_danger: Arc::new(AtomicBool::new(false)),
            danger_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            safe_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }
    
    /// Legacy constructor (keep for backward compatibility)
    pub async fn new(
        config: StoragePipelineConfig,
        repository: Arc<dyn StorageRepository>,
        _buffer: super::shared_buffer::SharedBuffer,
    ) -> Result<Self, String> {
        let (_tx, rx) = create_storage_channels(config.max_queue_size);
        let pipeline = Self::with_event_channel(config.clone(), repository, rx);
        
        if let Ok(last_seq) = pipeline.repository.get_last_stored_sequence().await {
            pipeline.last_stored_sequence.store(last_seq, Ordering::Relaxed);
        }
        
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
        self.handle = Some(qt_threading_utils::runtime::global_runtime().spawn(async move {
            pipeline.run_event_loop().await;
        }));
        
        Ok(())
    }

    /// Initialize last_stored_sequence from database (call before start)
    pub async fn initialize_sequence(&mut self) -> Result<(), String> {
        if let Ok(last_seq) = self.repository.get_last_stored_sequence().await {
            self.last_stored_sequence.store(last_seq, Ordering::Relaxed);
            tracing::info!("Storage last_seq initialized to {}", last_seq);
        }
        Ok(())
    }
    
    /// Event-driven main loop
    async fn run_event_loop(&self) {
        let mut flush_interval = tokio::time::interval(self.config.interval);
        let mut shutdown_rx = self.event_receiver.shutdown_rx.clone();
        let data_rx = self.event_receiver.data_rx.clone();
        let pending_batch = tokio::sync::Mutex::new(self.pending_batch.clone());
        let last_seq = self.last_stored_sequence.clone();
        let last_was_danger = self.last_was_danger.clone();
        let danger_count = self.danger_count.clone();
        let safe_count = self.safe_count.clone();
        let config = self.config.clone();
        let repository = self.repository.clone();
        
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
                            &last_was_danger,
                            &config,
                            &repository,
                            &danger_count,
                            &safe_count,
                        ).await;
                    }
                }
                
                _ = flush_interval.tick() => {
                    drop(data_rx_guard);
                    Self::flush_pending_batch(
                        &pending_batch, 
                        &last_seq, 
                        &config, 
                        &repository
                    ).await;
                }
            }
        }
        
        let batch_len = pending_batch.lock().await.len();
        tracing::info!("Draining {} pending records before shutdown", batch_len);
        Self::flush_pending_batch(&pending_batch, &last_seq, &config, &repository).await;
    }
    
    async fn handle_new_data(
        data_list: Vec<ProcessedData>,
        pending_batch: &tokio::sync::Mutex<Vec<ProcessedData>>,
        last_seq: &Arc<AtomicU64>,
        last_was_danger: &Arc<AtomicBool>,
        config: &StoragePipelineConfig,
        repository: &Arc<dyn StorageRepository>,
        danger_count: &Arc<std::sync::atomic::AtomicU32>,
        safe_count: &Arc<std::sync::atomic::AtomicU32>,
    ) {
        let last_seq_val = last_seq.load(Ordering::Acquire);
        
        for data in data_list {
            // Filter already stored
            if data.sequence_number > last_seq_val {
                // Handle alarm state transitions with debounce
                if data.is_danger {
                    // 危险状态：增加危险计数，重置安全计数
                    let current_danger_count = danger_count.fetch_add(1, Ordering::Relaxed) + 1;
                    safe_count.store(0, Ordering::Relaxed);
                    
                    // 检查是否达到防抖阈值
                    if config.alarm_debounce_count == 0 || current_danger_count >= config.alarm_debounce_count {
                        // Check if this is a NEW alarm (not continuous)
                        let expected = false;
                        if last_was_danger.compare_exchange(
                            expected, true, 
                            Ordering::Relaxed, 
                            Ordering::Relaxed
                        ).is_ok() {
                            // Transition: safe → danger, this is a NEW alarm
                            tracing::warn!(
                                "⚠️  NEW ALARM triggered at sequence {} (danger_count: {}, threshold: {})",
                                data.sequence_number,
                                current_danger_count,
                                config.alarm_debounce_count
                            );
                            Self::save_alarm_inner(data.clone(), config, repository).await;
                        }
                    }
                    // 移除高频 DEBUG 日志，避免日志刷屏
                } else {
                    // 安全状态：增加安全计数，重置危险计数
                    let current_safe_count = safe_count.fetch_add(1, Ordering::Relaxed) + 1;
                    danger_count.store(0, Ordering::Relaxed);
                    
                    // 检查是否达到解除防抖阈值
                    if config.alarm_clear_debounce_count == 0 || current_safe_count >= config.alarm_clear_debounce_count {
                        // Check if alarm was cleared
                        let expected = true;
                        if last_was_danger.compare_exchange(
                            expected, false,
                            Ordering::Relaxed,
                            Ordering::Relaxed
                        ).is_ok() {
                            tracing::info!(
                                "✅ Alarm CLEARED at sequence {} (safe_count: {}, threshold: {})",
                                data.sequence_number,
                                current_safe_count,
                                config.alarm_clear_debounce_count
                            );
                        }
                    }
                    // 移除高频 DEBUG 日志，避免日志刷屏
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
            Self::flush_pending_batch(pending_batch, last_seq, config, repository).await;
        }
    }
    
    async fn flush_pending_batch(
        pending_batch: &tokio::sync::Mutex<Vec<ProcessedData>>,
        last_seq: &Arc<AtomicU64>,
        config: &StoragePipelineConfig,
        repository: &Arc<dyn StorageRepository>,
    ) {
        let data_to_save = {
            let mut batch = pending_batch.lock().await;
            if batch.is_empty() {
                return;
            }
            std::mem::take(&mut *batch)
        };
        
        let max_seq = data_to_save.iter()
            .map(|d| d.sequence_number)
            .max()
            .unwrap_or(0);
        
        let max_records = config.max_records;
        let purge_threshold = config.purge_threshold;
        
        // Use retry with exponential backoff
        let result = with_retry(
            &RetryConfig {
                max_retries: config.max_retries,
                base_delay: config.retry_delay,
                ..Default::default()
            },
            || {
                let data = data_to_save.clone();
                let repo = Arc::clone(repository);
                async move {
                    repo.save_runtime_data_batch(&data).await
                        .map_err(|e| StorageError::Database(e.to_string()))
                }
            },
        ).await;
        
        match result {
            Ok(saved) => {
                tracing::info!("Saved {} records (seq <= {})", saved, max_seq);
                last_seq.store(max_seq, Ordering::Release);
                
                // Trigger purge in background
                if max_records > 0 {
                    let repo = Arc::clone(repository);
                    tokio::spawn(async move {
                        if let Err(e) = repo.purge_old_records(max_records, purge_threshold).await {
                            tracing::error!("Purge failed: {}", e);
                        }
                    });
                }
            }
            Err(e) => {
                tracing::error!("Failed to save batch after retries: {:?}", e);
                let mut batch = pending_batch.lock().await;
                batch.extend(data_to_save);
            }
        }
    }
    
    async fn save_alarm_inner(
        data: ProcessedData,
        config: &StoragePipelineConfig,
        repository: &Arc<dyn StorageRepository>,
    ) {
        let repo = Arc::clone(repository);
        let alarm_max = config.alarm_max_records;
        let alarm_purge = config.alarm_purge_threshold;
        
        tokio::spawn(async move {
            match repo.save_alarm_record(&data).await {
                Ok(alarm_id) => {
                    tracing::info!("Alarm saved with id: {}", alarm_id);
                    if alarm_max > 0 {
                        if let Err(e) = repo.purge_old_alarms(alarm_max, alarm_purge).await {
                            tracing::error!("Alarm purge failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to save alarm: {}", e);
                }
            }
        });
    }
    
    /// Save alarm asynchronously (public API for callbacks)
    pub fn save_alarm_async(&self, data: ProcessedData) {
        let config = self.config.clone();
        let repository = Arc::clone(&self.repository);
        tokio::spawn(async move {
            Self::save_alarm_inner(data, &config, &repository).await;
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
            repository: Arc::clone(&self.repository),
            event_receiver: StorageEventReceiver {
                data_rx: self.event_receiver.data_rx.clone(),
                shutdown_rx: self.event_receiver.shutdown_rx.clone(),
            },
            pending_batch: Vec::with_capacity(self.config.batch_size),
            last_stored_sequence: Arc::clone(&self.last_stored_sequence),
            running: Arc::clone(&self.running),
            handle: None,
            last_was_danger: Arc::clone(&self.last_was_danger),
            danger_count: Arc::clone(&self.danger_count),
            safe_count: Arc::clone(&self.safe_count),
        }
    }
    
    /// Notify danger cleared (legacy compatibility)
    pub fn notify_danger_cleared(&self) {
        self.last_was_danger.store(false, Ordering::Relaxed);
        tracing::debug!("Danger state reset to false");
    }
    
    /// Save alarm async (legacy public interface)
    /// 
    /// Note: This method is retained for backward compatibility.
    /// The internal alarm saving is now handled automatically via event handling.
    pub fn save_alarm_async_legacy(&self, data: ProcessedData) {
        // Check if we need to skip (continuous alarm logic)
        let last_was_danger = self.last_was_danger.load(Ordering::Relaxed);
        
        if last_was_danger {
            // Already in alarm state, skip
            tracing::debug!("Continuous alarm detected, skipping duplicate");
            return;
        }
        
        // Set alarm state
        self.last_was_danger.store(true, Ordering::Relaxed);
        
        // Spawn the async save
        let repo = Arc::clone(&self.repository);
        let alarm_max = self.config.alarm_max_records;
        let alarm_purge = self.config.alarm_purge_threshold;
        let sequence_number = data.sequence_number;
        
        tokio::spawn(async move {
            match repo.save_alarm_record(&data).await {
                Ok(alarm_id) => {
                    tracing::info!("New alarm saved with id: {} (sequence: {})", alarm_id, sequence_number);
                    
                    // Clean up old alarms
                    if alarm_max > 0 {
                        match repo.purge_old_alarms(alarm_max, alarm_purge).await {
                            Ok(purged) if purged > 0 => {
                                tracing::info!("Purged {} old alarms", purged);
                            }
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!("Failed to purge old alarms: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to save alarm record: {}", e);
                }
            }
        });
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
    use crate::repositories::mock_storage_repository::MockStorageRepository;
    use crate::pipeline::shared_buffer::ProcessedDataBuffer;
    use std::sync::RwLock;
    
    #[tokio::test]
    async fn test_new() {
        let repo = Arc::new(MockStorageRepository::new());
        let buffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));
        let config = StoragePipelineConfig::default();
        
        let pipeline = StoragePipeline::new(
            config,
            repo as Arc<dyn StorageRepository>,
            buffer,
        ).await;
        
        assert!(pipeline.is_ok());
    }
    
    #[tokio::test]
    async fn test_with_event_channel() {
        let repo = Arc::new(MockStorageRepository::new());
        let config = StoragePipelineConfig::default();
        let (_tx, rx) = create_storage_channels(100);
        
        let pipeline = StoragePipeline::with_event_channel(
            config,
            repo,
            rx,
        );
        
        assert_eq!(pipeline.queue_len(), 0);
        assert_eq!(pipeline.last_stored_sequence(), 0);
    }
    
    #[tokio::test]
    async fn test_start_stop() {
        let repo = Arc::new(MockStorageRepository::new());
        let config = StoragePipelineConfig::default();
        let (_tx, rx) = create_storage_channels(100);
        
        let mut pipeline = StoragePipeline::with_event_channel(
            config,
            repo,
            rx,
        );
        
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
        let config = StoragePipelineConfig::default();
        let (_tx, rx) = create_storage_channels(100);
        
        let pipeline = StoragePipeline::with_event_channel(
            config,
            repo,
            rx,
        );
        
        // Initial queue should be empty
        assert_eq!(pipeline.queue_len(), 0);
        assert_eq!(pipeline.last_stored_sequence(), 0);
    }
}
