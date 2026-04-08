// SensorData 存储管道

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use crate::config::pipeline_config::SensorDataStorageConfig;
use crate::models::SensorData;
use crate::repositories::sensor_data_repository::SensorDataRepository;
use crate::pipeline::sensor_data_event_channel::SensorDataEventReceiver;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

/// SensorData 存储管道
pub struct SensorStoragePipeline {
    config: SensorDataStorageConfig,
    repository: Arc<dyn SensorDataRepository>,
    event_receiver: SensorDataEventReceiver,
    pending_batch: Vec<SensorData>,
    last_flush: Instant,
    running: Arc<AtomicBool>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl SensorStoragePipeline {
    /// Create SensorStoragePipeline with event receiver
    pub fn with_event_channel(
        config: SensorDataStorageConfig,
        repository: Arc<dyn SensorDataRepository>,
        event_receiver: SensorDataEventReceiver,
    ) -> Self {
        let batch_size = config.batch_size;
        Self {
            config,
            repository,
            event_receiver,
            pending_batch: Vec::with_capacity(batch_size),
            last_flush: Instant::now(),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Start the sensor storage pipeline
    pub fn start(&mut self) -> Result<(), String> {
        // 如果已经在运行，先停止旧的任务
        // abort() 会立即终止任务，无需等待
        if self.running.load(Ordering::Relaxed) {
            warn!("传感器存储管道已在运行，先停止旧任务");
            self.stop();
        }

        self.running.store(true, Ordering::Relaxed);

        let pipeline = self.clone_for_callback();

        // 使用全局 tokio runtime 启动异步任务
        self.handle = Some(qt_threading_utils::runtime::global_runtime().spawn(async move {
            pipeline.run_event_loop().await;
        }));

        info!("传感器存储管道已启动");
        Ok(())
    }

    /// 停止传感器存储管道
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        // 发送关闭信号
        self.event_receiver.shutdown();

        // abort() 会立即终止 tokio 任务，无需阻塞等待
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    /// Main event loop
    async fn run_event_loop(&self) {
        let flush_interval = tokio::time::Duration::from_secs(self.config.interval_secs);
        let mut interval = tokio::time::interval(flush_interval);
        let mut shutdown_rx = self.event_receiver.shutdown_rx.clone();
        let data_rx = self.event_receiver.data_rx.clone();
        let pending_batch = Mutex::new(self.pending_batch.clone());
        let config = self.config.clone();
        let repository = self.repository.clone();
        let last_flush = Mutex::new(self.last_flush);

        loop {
            let mut data_rx_guard = data_rx.lock().await;
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("Sensor storage pipeline shutdown signal received");
                        break;
                    }
                }

                data = data_rx_guard.recv() => {
                    drop(data_rx_guard);
                    if let Some(data_list) = data {
                        let mut batch = pending_batch.lock().await;
                        for data in data_list {
                            batch.push(data);
                        }

                        // Check if batch is full and flush
                        if batch.len() >= config.batch_size {
                            drop(batch);
                            Self::flush_batch(&pending_batch, &last_flush, &config, &repository).await;
                        }
                    }
                }

                _ = interval.tick() => {
                    drop(data_rx_guard);
                    // Check if interval has elapsed and flush
                    let should_flush = {
                        let flush_guard = last_flush.lock().await;
                        flush_guard.elapsed() >= Duration::from_secs(config.interval_secs)
                    };
                    if should_flush {
                        Self::flush_batch(&pending_batch, &last_flush, &config, &repository).await;
                    }
                }
            }
        }

        // Drain remaining batch before shutdown
        let batch_len = pending_batch.lock().await.len();
        tracing::info!("Draining {} pending sensor data records before shutdown", batch_len);
        Self::flush_batch(&pending_batch, &last_flush, &config, &repository).await;
        
        // 确保在事件循环结束时重置 running 标志
        self.running.store(false, Ordering::Relaxed);
        tracing::debug!("Sensor storage pipeline event loop ended, running flag reset");
    }

    /// Flush pending batch to repository
    async fn flush_batch(
        pending_batch: &Mutex<Vec<SensorData>>,
        last_flush: &Mutex<Instant>,
        config: &SensorDataStorageConfig,
        repository: &Arc<dyn SensorDataRepository>,
    ) {
        let data_to_save = {
            let mut batch = pending_batch.lock().await;
            if batch.is_empty() {
                return;
            }
            std::mem::take(&mut *batch)
        };

        let _count = data_to_save.len();
        let max_records = config.max_records;

        // Save to repository
        match repository.save_sensor_data_batch(&data_to_save).await {
            Ok(saved) => {
                tracing::info!("Saved {} sensor data records", saved);

                // Update last_flush time
                let mut flush_guard = last_flush.lock().await;
                *flush_guard = Instant::now();
                drop(flush_guard);

                // Trigger purge in background if max_records is set
                if max_records > 0 {
                    let repo = Arc::clone(repository);
                    tokio::spawn(async move {
                        if let Err(e) = repo.purge_old_sensor_data(max_records).await {
                            tracing::error!("Sensor data purge failed: {}", e);
                        }
                    });
                }
            }
            Err(e) => {
                tracing::error!("Failed to save sensor data batch: {}", e);
                // Put data back into batch for retry
                let mut batch = pending_batch.lock().await;
                batch.extend(data_to_save);
            }
        }
    }

    /// Check if batch should be flushed
    #[allow(dead_code)]
    fn should_flush(&self) -> bool {
        let batch_full = self.pending_batch.len() >= self.config.batch_size;
        let interval_elapsed = self.last_flush.elapsed() >= Duration::from_secs(self.config.interval_secs);
        batch_full || interval_elapsed
    }

    /// Clone for callback
    fn clone_for_callback(&self) -> Self {
        Self {
            config: self.config.clone(),
            repository: Arc::clone(&self.repository),
            event_receiver: SensorDataEventReceiver {
                data_rx: self.event_receiver.data_rx.clone(),
                shutdown_rx: self.event_receiver.shutdown_rx.clone(),
            },
            pending_batch: Vec::with_capacity(self.config.batch_size),
            last_flush: self.last_flush,
            running: Arc::clone(&self.running),
            handle: None,
        }
    }
}

impl Drop for SensorStoragePipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::sensor_data_event_channel::create_sensor_data_channels;

    // Mock repository for testing
    struct MockSensorDataRepository;

    #[async_trait::async_trait]
    impl SensorDataRepository for MockSensorDataRepository {
        async fn save_sensor_data_batch(&self, data: &[SensorData]) -> Result<usize, String> {
            Ok(data.len())
        }

        async fn query_recent_sensor_data(&self, _limit: usize) -> Result<Vec<SensorData>, String> {
            Ok(vec![])
        }

        async fn get_latest_sensor_data(&self) -> Result<Option<SensorData>, String> {
            Ok(None)
        }

        async fn get_sensor_data_count(&self) -> Result<i64, String> {
            Ok(0)
        }

        async fn purge_old_sensor_data(&self, _max_records: usize) -> Result<usize, String> {
            Ok(0)
        }

        async fn health_check(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_with_event_channel() {
        let repo = Arc::new(MockSensorDataRepository);
        let config = SensorDataStorageConfig::default();
        let (_tx, rx) = create_sensor_data_channels(100);

        let pipeline = SensorStoragePipeline::with_event_channel(
            config,
            repo,
            rx,
        );

        // Verify initial state
        assert!(!pipeline.running.load(Ordering::Relaxed));
        assert!(pipeline.pending_batch.is_empty());
    }

    #[tokio::test]
    async fn test_start_stop() {
        let repo = Arc::new(MockSensorDataRepository);
        let config = SensorDataStorageConfig::default();
        let (_tx, rx) = create_sensor_data_channels(100);

        let mut pipeline = SensorStoragePipeline::with_event_channel(
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

    #[test]
    fn test_should_flush() {
        let repo = Arc::new(MockSensorDataRepository);
        let config = SensorDataStorageConfig {
            enabled: true,
            batch_size: 5,
            interval_secs: 60,
            max_records: 0,
        };
        let (_tx, rx) = create_sensor_data_channels(100);

        let mut pipeline = SensorStoragePipeline::with_event_channel(
            config.clone(),
            repo,
            rx,
        );

        // Initially should not flush
        assert!(!pipeline.should_flush());

        // Fill the batch
        pipeline.pending_batch = vec![
            SensorData::new(1.0, 2.0, 3.0),
            SensorData::new(4.0, 5.0, 6.0),
            SensorData::new(7.0, 8.0, 9.0),
            SensorData::new(10.0, 11.0, 12.0),
            SensorData::new(13.0, 14.0, 15.0),
        ];

        // Now should flush (batch full)
        assert!(pipeline.should_flush());
    }

    #[tokio::test]
    async fn test_flush_batch() {
        let repo: Arc<dyn SensorDataRepository> = Arc::new(MockSensorDataRepository);
        let config = SensorDataStorageConfig::default();
        let pending_batch = Mutex::new(vec![
            SensorData::new(1.0, 2.0, 3.0),
            SensorData::new(4.0, 5.0, 6.0),
        ]);
        let last_flush = Mutex::new(Instant::now());

        SensorStoragePipeline::flush_batch(
            &pending_batch,
            &last_flush,
            &config,
            &repo as &Arc<dyn SensorDataRepository>,
        ).await;

        // Batch should be empty after flush
        let batch = pending_batch.lock().await;
        assert!(batch.is_empty());
    }

    #[tokio::test]
    async fn test_flush_empty_batch() {
        let repo: Arc<dyn SensorDataRepository> = Arc::new(MockSensorDataRepository);
        let config = SensorDataStorageConfig::default();
        let pending_batch: Mutex<Vec<SensorData>> = Mutex::new(vec![]);
        let last_flush = Mutex::new(Instant::now());

        // Should not panic on empty batch
        SensorStoragePipeline::flush_batch(
            &pending_batch,
            &last_flush,
            &config,
            &repo as &Arc<dyn SensorDataRepository>,
        ).await;

        let batch = pending_batch.lock().await;
        assert!(batch.is_empty());
    }
}
