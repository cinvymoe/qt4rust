# 日志管理模块

全局日志管理系统，支持按模块/文件单独控制日志级别。

## 模块结构

```
src/logging/
├── mod.rs           # 模块入口，导出公共 API 和便捷宏
├── config.rs        # 日志配置结构和管理
├── filter.rs        # 日志系统初始化和过滤器
└── README.md        # 本文件
```

## 快速使用

### 1. 初始化日志系统

```rust
use qt_rust_demo::logging::init_logging_from_file;

fn main() {
    // 从配置文件初始化
    init_logging_from_file("config/logging.toml")
        .expect("Failed to initialize logging");
    
    // 应用程序代码...
}
```

### 2. 使用日志宏

```rust
use tracing::{trace, debug, info, warn, error};

fn my_function() {
    trace!("详细的调试信息");
    debug!("调试信息: value = {}", 42);
    info!("一般信息");
    warn!("警告信息");
    error!("错误信息");
}
```

### 3. 配置日志级别

编辑 `config/logging.toml`:

```toml
default_level = "info"
console_output = true
file_output = false

[[modules]]
module = "qt_rust_demo::pipeline::storage_pipeline"
level = "trace"

[[modules]]
module = "qt_rust_demo::pipeline::*"
level = "debug"
```

## API 文档

### 初始化函数

- `init_logging(config: Option<LogConfig>)` - 使用指定配置初始化
- `init_default_logging()` - 使用默认配置初始化
- `init_logging_from_file(path: &str)` - 从文件加载配置并初始化

### 配置结构

- `LogConfig` - 全局日志配置
- `LogLevel` - 日志级别枚举
- `ModuleLogLevel` - 模块日志级别配置

### 便捷宏

- `log_trace!()` - 带模块检查的 trace 日志
- `log_debug!()` - 带模块检查的 debug 日志
- `log_info!()` - 带模块检查的 info 日志
- `log_warn!()` - 带模块检查的 warn 日志
- `log_error!()` - 带模块检查的 error 日志

## 详细文档

参见 `doc/LOGGING_GUIDE.md` 获取完整使用指南。
