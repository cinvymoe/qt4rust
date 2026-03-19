// Tokio 运行时管理工具

use tokio::runtime::{Runtime, Handle};

/// 全局 Tokio 运行时管理器
/// 
/// 为 Qt 应用提供统一的异步运行时，避免创建多个运行时实例。
/// 
/// # 设计原则
/// 
/// - **单例模式**: 使用 `OnceLock` 确保全局只有一个运行时实例
/// - **多线程运行时**: 默认配置 4 个工作线程，支持并发任务
/// - **资源优化**: 所有组件共享同一个运行时，减少内存和线程开销
/// 
/// # 示例
/// 
/// ```rust
/// use qt_threading_utils::runtime::global_runtime;
/// 
/// // 获取全局运行时
/// let runtime = global_runtime();
/// 
/// // 生成异步任务
/// runtime.spawn(async {
///     println!("异步任务执行中...");
/// });
/// 
/// // 阻塞执行异步任务
/// runtime.block_on(async {
///     println!("阻塞执行异步任务");
/// });
/// ```
pub struct QtRuntime {
    runtime: Runtime,
}

impl QtRuntime {
    /// 创建新的运行时（多线程，4 个工作线程）
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("qt-tokio-worker")
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
        Self { runtime }
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

/// 全局运行时单例（使用 OnceLock）
/// 
/// 使用 `OnceLock` 实现线程安全的延迟初始化，确保全局只有一个运行时实例。
static GLOBAL_RUNTIME: std::sync::OnceLock<QtRuntime> = std::sync::OnceLock::new();

/// 获取全局运行时实例
/// 
/// 首次调用时会初始化运行时，后续调用直接返回已初始化的实例。
/// 
/// # 返回值
/// 
/// 返回全局运行时的静态引用，生命周期为 `'static`。
/// 
/// # 示例
/// 
/// ```rust
/// use qt_threading_utils::runtime::global_runtime;
/// 
/// let runtime = global_runtime();
/// runtime.spawn(async {
///     println!("Hello from global runtime!");
/// });
/// ```
pub fn global_runtime() -> &'static QtRuntime {
    GLOBAL_RUNTIME.get_or_init(|| QtRuntime::new())
}

/// 在全局运行时中执行异步任务（阻塞直到完成）
/// 
/// 这是一个便捷函数，等价于 `global_runtime().block_on(future)`。
/// 
/// # 参数
/// 
/// - `future`: 要执行的异步任务
/// 
/// # 返回值
/// 
/// 返回异步任务的输出结果。
/// 
/// # 注意
/// 
/// 此函数会阻塞当前线程直到异步任务完成，不应在异步上下文中调用。
/// 
/// # 示例
/// 
/// ```rust
/// use qt_threading_utils::runtime::block_on;
/// use std::time::Duration;
/// 
/// let result = block_on(async {
///     tokio::time::sleep(Duration::from_millis(100)).await;
///     42
/// });
/// 
/// assert_eq!(result, 42);
/// ```
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    global_runtime().block_on(future)
}

/// 在全局运行时中生成异步任务（非阻塞）
/// 
/// 这是一个便捷函数，等价于 `global_runtime().spawn(future)`。
/// 
/// # 参数
/// 
/// - `future`: 要生成的异步任务
/// 
/// # 返回值
/// 
/// 返回 `JoinHandle`，可用于等待任务完成或取消任务。
/// 
/// # 示例
/// 
/// ```rust
/// use qt_threading_utils::runtime::spawn;
/// 
/// let handle = spawn(async {
///     println!("后台任务执行中...");
///     42
/// });
/// 
/// // 可以稍后等待任务完成
/// // let result = handle.await.unwrap();
/// ```
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    global_runtime().spawn(future)
}

/// 在全局运行时中生成阻塞任务
/// 
/// 这是一个便捷函数，等价于 `global_runtime().spawn_blocking(f)`。
/// 
/// 阻塞任务会在专用的线程池中执行，不会阻塞异步工作线程。
/// 
/// # 参数
/// 
/// - `f`: 要执行的阻塞函数
/// 
/// # 返回值
/// 
/// 返回 `JoinHandle`，可用于等待任务完成。
/// 
/// # 使用场景
/// 
/// - 文件 I/O 操作
/// - 数据库查询
/// - CPU 密集型计算
/// - 调用同步 API
/// 
/// # 示例
/// 
/// ```rust
/// use qt_threading_utils::runtime::spawn_blocking;
/// 
/// let handle = spawn_blocking(|| {
///     // 执行阻塞操作
///     std::thread::sleep(std::time::Duration::from_secs(1));
///     "完成"
/// });
/// 
/// // 可以稍后等待任务完成
/// // let result = handle.await.unwrap();
/// ```
pub fn spawn_blocking<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    global_runtime().spawn_blocking(f)
}
