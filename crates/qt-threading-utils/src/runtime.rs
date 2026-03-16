// Tokio 运行时管理工具

use std::sync::Arc;
use tokio::runtime::{Runtime, Handle};

/// 全局 Tokio 运行时管理器
/// 
/// 为 Qt 应用提供统一的异步运行时，避免创建多个运行时实例
pub struct QtRuntime {
    runtime: Arc<Runtime>,
}

impl QtRuntime {
    /// 创建新的运行时（多线程）
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        Self {
            runtime: Arc::new(runtime),
        }
    }

    /// 创建单线程运行时
    pub fn new_current_thread() -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create current thread runtime");
        Self {
            runtime: Arc::new(runtime),
        }
    }

    /// 获取运行时句柄
    pub fn handle(&self) -> Handle {
        self.runtime.handle().clone()
    }

    /// 在运行时中执行异步任务（阻塞直到完成）
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.runtime.block_on(future)
    }

    /// 在运行时中生成异步任务（非阻塞）
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// 在运行时中生成阻塞任务
    pub fn spawn_blocking<F, R>(&self, f: F) -> tokio::task::JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.runtime.spawn_blocking(f)
    }
}

impl Default for QtRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局运行时单例（使用 lazy_static 或 once_cell）
static GLOBAL_RUNTIME: std::sync::OnceLock<QtRuntime> = std::sync::OnceLock::new();

/// 获取全局运行时实例
pub fn global_runtime() -> &'static QtRuntime {
    GLOBAL_RUNTIME.get_or_init(|| QtRuntime::new())
}

/// 在全局运行时中执行异步任务
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    global_runtime().block_on(future)
}

/// 在全局运行时中生成异步任务
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    global_runtime().spawn(future)
}

/// 在全局运行时中生成阻塞任务
pub fn spawn_blocking<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    global_runtime().spawn_blocking(f)
}
