# qt-threading-utils

基于 Tokio 的 Qt 应用线程和异步任务管理工具库。

## 功能特性

- **异步运行时管理**: 提供全局 Tokio 运行时，统一管理异步任务
- **周期定时器**: 使用 Tokio 实现的高效周期任务调度
- **单次定时器**: 延迟执行一次性任务
- **数据采集器**: 后台异步数据采集框架
- **阻塞式 API**: 为 Qt 信号槽提供同步接口
- **线程安全**: 所有组件都是线程安全的

## 依赖

```toml
[dependencies]
tokio = { version = "1.42", features = ["rt", "rt-multi-thread", "time", "sync", "macros"] }
futures = "0.3"
```

## 使用示例

### 1. 全局运行时

```rust
use qt_threading_utils::prelude::*;

// 获取全局运行时
let runtime = global_runtime();

// 执行异步任务（阻塞）
runtime.block_on(async {
    println!("异步任务执行中...");
});

// 生成异步任务（非阻塞）
runtime.spawn(async {
    println!("后台任务执行中...");
});

// 或使用便捷函数
block_on(async {
    println!("使用全局函数执行异步任务");
});
```

### 2. 异步周期定时器

```rust
use qt_threading_utils::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let timer = PeriodicTimer::new(Duration::from_secs(1));
    
    // 启动定时器
    timer.start(|| {
        println!("定时器触发！");
    }).await;
    
    // 等待一段时间
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // 停止定时器
    timer.stop().await;
}
```

### 3. 阻塞式周期定时器（适用于 Qt）

```rust
use qt_threading_utils::prelude::*;
use std::time::Duration;

// 在 Qt 信号槽或其他同步代码中使用
fn setup_timer() {
    let timer = BlockingPeriodicTimer::new(Duration::from_millis(100));
    
    // 启动定时器（阻塞调用）
    timer.start(|| {
        println!("定时器触发！");
    });
    
    // 稍后停止
    std::thread::sleep(Duration::from_secs(2));
    timer.stop();
}
```

### 4. 异步数据采集器

```rust
use qt_threading_utils::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let collector = DataCollector::new(Duration::from_millis(500));
    
    // 启动采集
    collector.start(|| {
        println!("采集数据...");
    }).await;
    
    // 检查状态
    if collector.is_running().await {
        println!("采集器正在运行");
    }
    
    // 停止采集
    collector.stop().await;
}
```

### 5. 阻塞式数据采集器（适用于 Qt）

```rust
use qt_threading_utils::prelude::*;
use std::time::Duration;

fn setup_data_collector() {
    let collector = BlockingDataCollector::new(Duration::from_millis(100));
    
    // 启动采集（阻塞调用）
    collector.start(|| {
        println!("采集传感器数据...");
    });
    
    // 检查状态
    if collector.is_running() {
        println!("采集器正在运行");
    }
    
    // 停止采集
    collector.stop();
}
```

### 6. 单次定时器

```rust
use qt_threading_utils::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let timer = OneShotTimer::new(Duration::from_secs(2));
    
    // 延迟执行
    timer.start(|| {
        println!("延迟任务执行！");
    }).await;
}
```

### 7. 在 Qt 应用中集成

```rust
use qt_threading_utils::prelude::*;
use std::time::Duration;

// 在 Qt 应用初始化时设置
fn init_qt_app() {
    // 使用全局运行时
    let runtime = global_runtime();
    
    // 创建阻塞式采集器用于传感器数据
    let sensor_collector = BlockingDataCollector::new(Duration::from_millis(50));
    
    sensor_collector.start(|| {
        // 读取传感器并更新 Qt 模型
        println!("更新传感器数据到 Qt 模型");
    });
    
    // 创建定时器用于 UI 更新
    let ui_timer = BlockingPeriodicTimer::new(Duration::from_millis(16));
    ui_timer.start(|| {
        // 更新 UI（约 60 FPS）
        println!("更新 UI");
    });
}
```

## 应用场景

- Qt 应用后台异步任务管理
- 定时数据采集和处理
- 传感器数据轮询
- 周期性 UI 更新
- 网络请求和 I/O 操作
- 与 Qt 信号槽系统集成

## 设计原则

1. **异步优先**: 核心 API 使用 async/await，提供最佳性能
2. **阻塞兼容**: 提供阻塞式 API 用于 Qt 同步代码
3. **全局运行时**: 避免创建多个运行时实例，统一管理
4. **线程安全**: 所有组件都可以安全地在多线程环境中使用
5. **零成本抽象**: 尽可能减少运行时开销

## 注意事项

- 阻塞式 API 内部使用 `block_on`，不应在异步上下文中调用
- 全局运行时在首次访问时初始化，之后复用
- 定时器和采集器在 Drop 时会自动停止
