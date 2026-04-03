// 采集管道（异步版本）

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tokio::task::JoinHandle;
use crate::repositories::CraneDataRepository;
use crate::models::{ProcessedData, SensorData};
use super::shared_buffer::SharedBuffer;
use super::event_channel::StorageEventSender;
use super::sensor_data_event_channel::SensorDataEventSender;
use super::filter_buffer::FilterBuffer;

/// 采集管道配置
#[derive(Debug, Clone)]
pub struct CollectionPipelineConfig {
    /// 采集间隔
    pub interval: Duration,
    
    /// 失败重试次数
    pub max_retries: u32,
    
    /// 重试延迟
    pub retry_delay: Duration,
    
    /// 断连检测阈值（连续失败次数）
    pub disconnect_threshold: u32,
    
    /// 是否启用 panic 恢复
    pub enable_panic_recovery: bool,
    
    /// 最大重启次数
    pub max_restarts: usize,
}

impl Default for CollectionPipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),
            max_retries: 3,
            retry_delay: Duration::from_millis(10),
            disconnect_threshold: 10,
            enable_panic_recovery: true,
            max_restarts: 5,
        }
    }
}

/// 采集管道（异步版本）
pub struct CollectionPipeline {
    config: CollectionPipelineConfig,
    repository: Arc<CraneDataRepository>,
    display_buffer: SharedBuffer,
    filter_buffer: Option<Arc<Mutex<FilterBuffer>>>,
    running: Arc<AtomicBool>,
    sequence_number: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
    alarm_callback: Option<Arc<dyn Fn(ProcessedData) + Send + Sync>>,
    danger_cleared_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    storage_event_sender: Option<StorageEventSender>,
    sensor_storage_sender: Option<SensorDataEventSender>,
}

impl CollectionPipeline {
    /// 创建新的采集管道
    pub fn new(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        display_buffer: SharedBuffer,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer,
            filter_buffer: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: None,
            sensor_storage_sender: None,
        }
    }

    /// 创建采集管道并附带事件发送器
    pub fn with_event_sender(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        display_buffer: SharedBuffer,
        storage_event_sender: StorageEventSender,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer,
            filter_buffer: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: Some(storage_event_sender),
            sensor_storage_sender: None,
        }
    }

    /// 创建采集管道并附带存储发送器和传感器存储发送器
    pub fn with_storage_sender(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        display_buffer: SharedBuffer,
        storage_event_sender: StorageEventSender,
        sensor_storage_sender: SensorDataEventSender,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer,
            filter_buffer: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: Some(storage_event_sender),
            sensor_storage_sender: Some(sensor_storage_sender),
        }
    }

    /// 创建采集管道（写入滤波缓冲区，用于多速率架构）
    pub fn with_filter_buffer(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        filter_buffer: Arc<Mutex<FilterBuffer>>,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer: Arc::new(std::sync::RwLock::new(
                super::shared_buffer::ProcessedDataBuffer::new(100)
            )),
            filter_buffer: Some(filter_buffer),
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: None,
            sensor_storage_sender: None,
        }
    }
    
    /// 设置报警回调
    /// 
    /// 当检测到报警状态时，会调用此回调函数
    pub fn set_alarm_callback<F>(&mut self, callback: F)
    where
        F: Fn(ProcessedData) + Send + Sync + 'static,
    {
        self.alarm_callback = Some(Arc::new(callback));
    }
    
    /// 设置报警解除回调
    /// 
    /// 当报警状态解除时（is_danger 从 true 变为 false），会调用此回调函数
    pub fn set_danger_cleared_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.danger_cleared_callback = Some(Arc::new(callback));
    }
    
    /// 设置初始序列号
    /// 
    /// 用于在程序重启后继续之前的序列号
    pub fn set_initial_sequence(&mut self, sequence: u64) {
        self.sequence_number.store(sequence, Ordering::Relaxed);
        tracing::info!(" Collection pipeline sequence initialized to {}", sequence);
    }
    
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            tracing::warn!(" Collection pipeline already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);

        let config = self.config.clone();
        let repository = Arc::clone(&self.repository);
        let display_buffer = Arc::clone(&self.display_buffer);
        let filter_buffer = self.filter_buffer.clone();
        let running = Arc::clone(&self.running);
        let sequence_number = Arc::clone(&self.sequence_number);
        let alarm_callback = self.alarm_callback.clone();
        let danger_cleared_callback = self.danger_cleared_callback.clone();
        let storage_event_sender = self.storage_event_sender.clone();
        let sensor_storage_sender = self.sensor_storage_sender.clone();

        let handle = qt_threading_utils::runtime::global_runtime().spawn(async move {
            tracing::info!(" Collection pipeline started (mode: {})", 
                if filter_buffer.is_some() { "filter" } else { "legacy" });
            let mut consecutive_failures = 0;
            let mut interval_timer = tokio::time::interval(config.interval);
            let mut previous_danger = false;

            while running.load(Ordering::Relaxed) {
                interval_timer.tick().await;

                let repo = Arc::clone(&repository);
                let cfg = config.clone();
                let result = tokio::task::spawn_blocking(move || {
                    Self::collect_with_retry(&repo, &cfg)
                }).await;

                match result {
                    Ok(Ok(sensor_data)) => {
                        consecutive_failures = 0;

                        if let Some(ref fb) = filter_buffer {
                            // 多速率模式: 只写入滤波缓冲区
                            if let Ok(mut fb_guard) = fb.lock() {
                                fb_guard.push(sensor_data);
                            }
                        } else {
                            // 遗留模式: 处理数据并写入显示缓冲区和存储
                            let seq = sequence_number.fetch_add(1, Ordering::Relaxed);
                            let processed = ProcessedData::from_sensor_data(sensor_data.clone(), seq);

                            // Send raw sensor data to storage pipeline if configured
                            if let Some(ref sender) = sensor_storage_sender {
                                if let Err(e) = sender.try_send_data(vec![sensor_data]) {
                                    tracing::warn!("Failed to send sensor data to storage: {}", e);
                                }
                            }

                            if let Some(ref sender) = storage_event_sender {
                                if let Err(e) = sender.try_send_data(vec![processed.clone()]) {
                                    tracing::warn!("Failed to send data to storage: {}", e);
                                }
                            }

                            let current_danger = processed.is_danger;

                            if current_danger && !previous_danger {
                                tracing::warn!("[ALARM] Danger detected! Moment: {:.1}%", processed.moment_percentage);
                                if let Some(ref callback) = alarm_callback {
                                    callback(processed.clone());
                                }
                            } else if !current_danger && previous_danger {
                                tracing::info!("[ALARM] Danger cleared");
                                if let Some(ref callback) = danger_cleared_callback {
                                    callback();
                                }
                            }

                            previous_danger = current_danger;

                            match display_buffer.try_write() {
                                Ok(mut buf) => { buf.push(processed); }
                                Err(_) => { tracing::warn!(" Failed to acquire buffer lock, skipping write"); }
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        consecutive_failures += 1;
                        tracing::error!(" Collection failed: {} (consecutive: {})", e, consecutive_failures);
                        
                        if filter_buffer.is_none() {
                            match display_buffer.try_write() {
                                Ok(mut buf) => { buf.record_error(); }
                                Err(_) => {}
                            }
                        }
                        
                        if consecutive_failures >= config.disconnect_threshold {
                            tracing::error!(" Sensor disconnected (threshold reached)");
                        }
                    }
                    Err(e) => {
                        if e.is_panic() {
                            tracing::error!("[PANIC] Collection task panicked: {:?}", e);
                        } else {
                            tracing::error!(" Collection task cancelled: {}", e);
                        }
                    }
                }
            }
            
            tracing::info!(" Collection pipeline stopped");
        });
        
        self.handle = Some(handle);
    }
    
    /// 停止管道
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            qt_threading_utils::runtime::global_runtime().block_on(async {
                let _ = handle.await;
            });
        }
    }
    
    /// 带重试的数据采集
    fn collect_with_retry(
        repository: &CraneDataRepository,
        config: &CollectionPipelineConfig,
    ) -> Result<SensorData, String> {
        let mut last_error = String::new();
        
        for attempt in 0..=config.max_retries {
            match repository.get_latest_sensor_data() {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = e;
                    if attempt < config.max_retries {
                        std::thread::sleep(config.retry_delay);
                    }
                }
            }
        }
        
        Err(format!("Failed after {} retries: {}", config.max_retries, last_error))
    }
}

impl Drop for CollectionPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;
    
    #[test]
    fn test_config_default() {
        let config = CollectionPipelineConfig::default();
        
        assert_eq!(config.interval, Duration::from_millis(100));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_millis(10));
        assert_eq!(config.disconnect_threshold, 10);
        assert!(config.enable_panic_recovery);
        assert_eq!(config.max_restarts, 5);
    }
    
    #[test]
    fn test_pipeline_creation() {
        use crate::config::config_manager::ConfigManager;
        
        let config = CollectionPipelineConfig::default();
        let config_manager = Arc::new(ConfigManager::default());
        let repository = Arc::new(CraneDataRepository::new(config_manager));
        let buffer = Arc::new(RwLock::new(
            super::super::shared_buffer::ProcessedDataBuffer::new(100)
        ));
        
        let pipeline = CollectionPipeline::new(config, repository, buffer);
        
        assert!(!pipeline.running.load(Ordering::Relaxed));
    }
}
