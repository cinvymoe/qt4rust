// Timer utilities - 基于 Tokio 的定时器

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// 周期定时器 - 使用 Tokio 异步执行周期任务
pub struct PeriodicTimer {
    interval: Duration,
    running: Arc<RwLock<bool>>,
    handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl PeriodicTimer {
    /// 创建新的周期定时器
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            running: Arc::new(RwLock::new(false)),
            handle: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动定时器（异步）
    pub async fn start<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut running = self.running.write().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);

        let running_clone = Arc::clone(&self.running);
        let interval = self.interval;
        let callback = Arc::new(callback);

        let handle = tokio::spawn(async move {
            loop {
                let is_running = *running_clone.read().await;
                if !is_running {
                    break;
                }

                callback();
                tokio::time::sleep(interval).await;
            }
        });

        let mut handle_lock = self.handle.write().await;
        *handle_lock = Some(handle);
    }

    /// 停止定时器（异步）
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        let mut handle_lock = self.handle.write().await;
        if let Some(handle) = handle_lock.take() {
            let _ = handle.await;
        }
    }

    /// 获取定时器间隔
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// 检查是否正在运行
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

/// 单次定时器 - 延迟执行一次任务
pub struct OneShotTimer {
    delay: Duration,
}

impl OneShotTimer {
    /// 创建新的单次定时器
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }

    /// 启动定时器并执行回调（异步）
    pub async fn start<F>(self, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        tokio::time::sleep(self.delay).await;
        callback();
    }

    /// 获取延迟时间
    pub fn delay(&self) -> Duration {
        self.delay
    }
}

/// 阻塞式周期定时器 - 使用全局运行时
/// 
/// 适用于 Qt 信号槽等同步场景
pub struct BlockingPeriodicTimer {
    interval: Duration,
    running: Arc<std::sync::atomic::AtomicBool>,
    handle: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
}

impl BlockingPeriodicTimer {
    /// 创建新的阻塞式周期定时器
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// 启动定时器（阻塞调用）
    /// 
    /// 使用全局运行时生成任务
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
                
                // 捕获 panic
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    callback();
                }));
                
                if let Err(e) = result {
                    eprintln!("[ERROR] Timer callback panicked: {:?}", e);
                }
            }
        });

        if let Ok(mut handle_lock) = self.handle.lock() {
            *handle_lock = Some(handle);
        }
    }

    /// 停止定时器（阻塞调用）
    pub fn stop(&self) {
        use std::sync::atomic::Ordering;
        
        self.running.store(false, Ordering::Relaxed);

        if let Ok(mut handle_lock) = self.handle.lock() {
            if let Some(handle) = handle_lock.take() {
                crate::runtime::global_runtime().block_on(async {
                    let _ = handle.await;
                });
            }
        }
    }

    /// 获取定时器间隔
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for BlockingPeriodicTimer {
    fn drop(&mut self) {
        self.stop();
    }
}
