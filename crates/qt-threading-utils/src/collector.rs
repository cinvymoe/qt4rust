// Data collector utilities - 基于 Tokio 的异步数据采集器

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// 数据采集器 - 使用 Tokio 在后台异步执行定期任务
/// 
/// # 示例
/// ```rust
/// use qt_threading_utils::prelude::*;
/// use std::time::Duration;
/// 
/// let collector = DataCollector::new(Duration::from_secs(1));
/// 
/// // 在异步上下文中使用
/// collector.start(|| {
///     println!("采集数据...");
/// }).await;
/// 
/// // ... 执行其他操作
/// collector.stop().await;
/// ```
pub struct DataCollector {
    interval: Duration,
    running: Arc<RwLock<bool>>,
    handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl DataCollector {
    /// 创建新的数据采集器
    /// 
    /// # 参数
    /// - `interval`: 采集间隔
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            running: Arc::new(RwLock::new(false)),
            handle: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动数据采集（异步）
    pub async fn start<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut running = self.running.write().await;
        if *running {
            return; // 已经在运行
        }
        *running = true;
        drop(running);

        let running_clone = Arc::clone(&self.running);
        let interval = self.interval;
        let callback = Arc::new(callback);

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                let is_running = *running_clone.read().await;
                if !is_running {
                    break;
                }

                interval_timer.tick().await;
                callback();
            }
        });

        let mut handle_lock = self.handle.write().await;
        *handle_lock = Some(handle);
    }

    /// 停止数据采集（异步）
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        let mut handle_lock = self.handle.write().await;
        if let Some(handle) = handle_lock.take() {
            let _ = handle.await;
        }
    }

    /// 获取采集间隔
    pub fn interval(&self) -> Duration {
        self.interval
    }
    
    /// 检查是否正在运行（异步）
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

/// 同步版本的数据采集器 - 使用全局运行时
/// 
/// 适用于不能使用 async/await 的场景（如 Qt 信号槽）
/// 
/// # 重要
/// 使用全局运行时，避免创建多个运行时实例导致资源浪费
pub struct BlockingDataCollector {
    interval: Duration,
    running: Arc<std::sync::atomic::AtomicBool>,
    handle: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
}

impl BlockingDataCollector {
    /// 创建新的阻塞式数据采集器
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// 启动数据采集（阻塞调用）
    /// 
    /// 使用全局运行时生成任务，避免创建新的运行时
    pub fn start<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        use std::sync::atomic::Ordering;
        
        if self.running.load(Ordering::Relaxed) {
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);

        let running_clone = Arc::clone(&self.running);
        let interval = self.interval;
        let callback = Arc::new(callback);

        // 使用全局运行时生成任务
        let handle = crate::runtime::global_runtime().spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                if !running_clone.load(Ordering::Relaxed) {
                    break;
                }

                interval_timer.tick().await;
                
                // 捕获 panic 以防止任务崩溃
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    callback();
                }));
                
                if let Err(e) = result {
                    eprintln!("[ERROR] Collector callback panicked: {:?}", e);
                    // 继续运行，不中断采集
                }
            }
        });

        if let Ok(mut handle_lock) = self.handle.lock() {
            *handle_lock = Some(handle);
        }
    }

    /// 停止数据采集（阻塞调用）
    pub fn stop(&self) {
        use std::sync::atomic::Ordering;
        
        self.running.store(false, Ordering::Relaxed);

        if let Ok(mut handle_lock) = self.handle.lock() {
            if let Some(handle) = handle_lock.take() {
                // 使用全局运行时等待任务完成
                crate::runtime::global_runtime().block_on(async {
                    let _ = handle.await;
                });
            }
        }
    }

    /// 获取采集间隔
    pub fn interval(&self) -> Duration {
        self.interval
    }
    
    /// 检查是否正在运行（阻塞调用）
    pub fn is_running(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for BlockingDataCollector {
    fn drop(&mut self) {
        self.stop();
    }
}
