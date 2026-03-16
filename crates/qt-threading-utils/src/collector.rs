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
/// let collector = DataCollector::new(Duration::from_secs(1), || {
///     println!("采集数据...");
/// });
/// 
/// collector.start();
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

/// 同步版本的数据采集器 - 使用阻塞调用
/// 
/// 适用于不能使用 async/await 的场景（如 Qt 信号槽）
pub struct BlockingDataCollector {
    interval: Duration,
    running: Arc<RwLock<bool>>,
    handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl BlockingDataCollector {
    /// 创建新的阻塞式数据采集器
    pub fn new(interval: Duration) -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        
        Self {
            interval,
            running: Arc::new(RwLock::new(false)),
            handle: Arc::new(RwLock::new(None)),
            runtime: Arc::new(runtime),
        }
    }

    /// 启动数据采集（阻塞调用）
    pub fn start<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.runtime.block_on(async {
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
        });
    }

    /// 停止数据采集（阻塞调用）
    pub fn stop(&self) {
        self.runtime.block_on(async {
            let mut running = self.running.write().await;
            *running = false;
            drop(running);

            let mut handle_lock = self.handle.write().await;
            if let Some(handle) = handle_lock.take() {
                let _ = handle.await;
            }
        });
    }

    /// 获取采集间隔
    pub fn interval(&self) -> Duration {
        self.interval
    }
    
    /// 检查是否正在运行（阻塞调用）
    pub fn is_running(&self) -> bool {
        self.runtime.block_on(async {
            *self.running.read().await
        })
    }
}

impl Drop for BlockingDataCollector {
    fn drop(&mut self) {
        self.stop();
    }
}
