# qt-threading-utils

Qt 应用的线程和定时器工具库。

## 功能特性

- **定时器**: 周期定时器和单次定时器
- **数据采集器**: 后台数据采集框架
- **线程安全**: 提供线程安全的工具

## 使用示例

```rust
use qt_threading_utils::prelude::*;
use std::time::Duration;

// 创建周期定时器
let timer = PeriodicTimer::new(Duration::from_millis(100));

// 创建数据采集器
let collector = DataCollector::new(Duration::from_millis(100));
```

## 应用场景

- Qt 应用后台任务
- 定时数据采集
- 传感器数据轮询
