// 采集管道（异步版本）

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::task::JoinHandle;
use crate::repositories::CraneDataRepository;
use crate::models::{ProcessedData, SensorData};
use super::shared_buffer::SharedBuffer;

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
    buffer: SharedBuffer,
    running: Arc<AtomicBool>,
    sequence_number: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
    /// 报警回调（可选）
    alarm_callback: Option<Arc<dyn Fn(ProcessedData) + Send + Sync>>,
}

impl CollectionPipeline {
    /// 创建新的采集管道
    pub fn new(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        buffer: SharedBuffer,
    ) -> Self {
        Self {
            config,
            repository,
            buffer,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
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
    
    /// 启动管道
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            eprintln!("[WARN] Collection pipeline already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let config = self.config.clone();
        let repository = Arc::clone(&self.repository);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let sequence_number = Arc::clone(&self.sequence_number);
        let alarm_callback = self.alarm_callback.clone();  // 克隆回调
        
        // 使用全局运行时生成任务
        let handle = qt_threading_utils::runtime::global_runtime().spawn(async move {
            eprintln!("[INFO] Collection pipeline started");
            let mut consecutive_failures = 0;
            let mut interval_timer = tokio::time::interval(config.interval);
            
            while running.load(Ordering::Relaxed) {
                interval_timer.tick().await;
                
                // 在阻塞线程池中执行同步采集
                let repo = Arc::clone(&repository);
                let cfg = config.clone();
                let result = tokio::task::spawn_blocking(move || {
                    Self::collect_with_retry(&repo, &cfg)
                }).await;
                
                match result {
                    Ok(Ok(sensor_data)) => {
                        // 采集成功
                        consecutive_failures = 0;
                        
                        let seq = sequence_number.fetch_add(1, Ordering::Relaxed);
                        let processed = ProcessedData::from_sensor_data(sensor_data, seq);
                        
                        // 检测报警状态，触发回调
                        if processed.is_danger {
                            if let Some(ref callback) = alarm_callback {
                                eprintln!("[ALARM] Danger detected! Moment: {:.1}%", processed.moment_percentage);
                                callback(processed.clone());
                            }
                        }
                        
                        // 使用 try_write 避免死锁
                        match buffer.try_write() {
                            Ok(mut buf) => {
                                buf.push(processed);
                            }
                            Err(_) => {
                                eprintln!("[WARN] Failed to acquire buffer lock, skipping write");
                                // 跳过本次写入，避免阻塞
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        // 采集失败
                        consecutive_failures += 1;
                        eprintln!("[ERROR] Collection failed: {} (consecutive: {})", 
                                  e, consecutive_failures);
                        
                        match buffer.try_write() {
                            Ok(mut buf) => {
                                buf.record_error();
                            }
                            Err(_) => {
                                eprintln!("[WARN] Failed to acquire buffer lock for error recording");
                            }
                        }
                        
                        // 检测断连
                        if consecutive_failures >= config.disconnect_threshold {
                            eprintln!("[ERROR] Sensor disconnected (threshold reached)");
                            // TODO: 触发断连事件
                        }
                    }
                    Err(e) => {
                        // 任务 panic 检测
                        if e.is_panic() {
                            eprintln!("[PANIC] Collection task panicked: {:?}", e);
                            // 记录 panic 信息到 stderr
                            let panic_payload = e.into_panic();
                            if let Some(panic_msg) = panic_payload.downcast_ref::<&str>() {
                                eprintln!("[PANIC] Panic message: {}", panic_msg);
                            } else if let Some(panic_msg) = panic_payload.downcast_ref::<String>() {
                                eprintln!("[PANIC] Panic message: {}", panic_msg);
                            } else {
                                eprintln!("[PANIC] Panic message: <unknown type>");
                            }
                        } else {
                            eprintln!("[ERROR] Collection task cancelled: {}", e);
                        }
                        
                        // 在 buffer 中记录错误
                        match buffer.try_write() {
                            Ok(mut buf) => {
                                buf.record_error();
                            }
                            Err(_) => {
                                eprintln!("[WARN] Failed to acquire buffer lock for panic error recording");
                            }
                        }
                    }
                }
            }
            
            eprintln!("[INFO] Collection pipeline stopped");
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
