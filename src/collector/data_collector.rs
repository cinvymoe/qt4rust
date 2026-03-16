// 后台数据采集器 - 简化版本，直接使用 std::thread

use crate::repositories::CraneDataRepository;
use crate::intents::MonitoringIntent;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// 数据采集器
pub struct DataCollector {
    /// 数据仓库
    repository: Arc<Mutex<CraneDataRepository>>,
    
    /// 运行标志
    running: Arc<AtomicBool>,
    
    /// 线程句柄
    handle: Option<thread::JoinHandle<()>>,
}

impl DataCollector {
    /// 创建新的数据采集器
    pub fn new(repository: CraneDataRepository) -> Self {
        Self {
            repository: Arc::new(Mutex::new(repository)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
    
    /// 启动数据采集（100ms 间隔）
    pub fn start<F>(&mut self, on_data: F)
    where
        F: Fn(MonitoringIntent) + Send + 'static,
    {
        if self.is_running() {
            eprintln!("[WARN] Data collector is already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let repo = Arc::clone(&self.repository);
        let running = Arc::clone(&self.running);
        
        let handle = thread::spawn(move || {
            eprintln!("[INFO] Data collector thread started");
            
            while running.load(Ordering::Relaxed) {
                // 数据采集回调
                if let Ok(repository) = repo.lock() {
                    match repository.get_latest_sensor_data() {
                        Ok(sensor_data) => {
                            // 数据采集成功，触发回调
                            on_data(MonitoringIntent::SensorDataUpdated(sensor_data));
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Failed to collect sensor data: {}", e);
                            on_data(MonitoringIntent::SensorDisconnected);
                        }
                    }
                }
                
                // 100ms 采集间隔
                thread::sleep(Duration::from_millis(100));
            }
            
            eprintln!("[INFO] Data collector thread stopped");
        });
        
        self.handle = Some(handle);
        eprintln!("[INFO] Data collector started (100ms interval)");
    }
    
    /// 停止数据采集
    pub fn stop(&mut self) {
        if !self.is_running() {
            return;
        }
        
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
            eprintln!("[INFO] Data collector stopped");
        }
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for DataCollector {
    fn drop(&mut self) {
        self.stop();
    }
}
