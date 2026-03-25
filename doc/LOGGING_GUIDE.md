# 日志管理系统使用指南

## 概述

本项目实现了一个全局的日志管理系统，可以对每个模块/文件的日志输出进行单独控制。

## 特性

- ✅ 按模块/文件单独控制日志级别
- ✅ 支持通配符匹配（`*`）
- ✅ 支持控制台和文件输出
- ✅ 提供便捷的日志宏
- ✅ 运行时可配置
- ✅ 支持环境变量覆盖

## 快速开始

### 1. 初始化日志系统

在应用程序入口（`main.rs` 或 `lib.rs`）初始化日志系统：

```rust
use qt_rust_demo::logging::init_logging_from_file;

fn main() {
    // 从配置文件初始化
    init_logging_from_file("config/logging.toml")
        .expect("Failed to initialize logging");
    
    // 或使用默认配置
    // qt_rust_demo::logging::init_default_logging();
    
    // 应用程序代码...
}
```

### 2. 配置日志级别

编辑 `config/logging.toml` 文件：

```toml
# 默认日志级别
default_level = "info"

# 是否输出到控制台
console_output = true

# 是否输出到文件
file_output = false

# 日志文件路径
log_file = "logs/app.log"

# 各模块的日志级别配置
[[modules]]
module = "qt_rust_demo::pipeline::storage_pipeline"
level = "trace"  # 最详细

[[modules]]
module = "qt_rust_demo::pipeline::*"
level = "debug"

[[modules]]
module = "qt_rust_demo::repositories::*"
level = "info"
```

### 3. 在代码中使用日志

#### 方式一：使用标准 tracing 宏（推荐）

```rust
use tracing::{trace, debug, info, warn, error};

pub fn my_function() {
    trace!("详细的调试信息");
    debug!("调试信息: value = {}", 42);
    info!("一般信息");
    warn!("警告信息");
    error!("错误信息");
}
```

#### 方式二：使用自定义日志宏（带模块级别检查）

```rust
use qt_rust_demo::{log_trace, log_debug, log_info, log_warn, log_error};

pub fn my_function() {
    log_trace!("详细的调试信息");
    log_debug!("调试信息: value = {}", 42);
    log_info!("一般信息");
    log_warn!("警告信息");
    log_error!("错误信息");
}
```

## 日志级别说明

| 级别 | 说明 | 使用场景 |
|------|------|---------|
| `trace` | 最详细的日志 | 函数调用、变量值、详细流程 |
| `debug` | 调试信息 | 开发调试、问题排查 |
| `info` | 一般信息 | 重要操作、状态变化 |
| `warn` | 警告信息 | 潜在问题、异常情况 |
| `error` | 错误信息 | 错误、失败操作 |
| `off` | 关闭日志 | 完全禁用某个模块的日志 |

## 配置示例

### 示例 1: 调试特定模块

```toml
default_level = "warn"  # 默认只显示警告和错误

[[modules]]
module = "qt_rust_demo::pipeline::storage_pipeline"
level = "trace"  # 只有这个文件显示所有日志
```

### 示例 2: 生产环境配置

```toml
default_level = "info"
console_output = true
file_output = true
log_file = "logs/production.log"

[[modules]]
module = "qt_rust_demo::pipeline::*"
level = "warn"  # Pipeline 只记录警告和错误

[[modules]]
module = "qt_rust_demo::repositories::*"
level = "error"  # Repository 只记录错误
```

### 示例 3: 开发环境配置

```toml
default_level = "debug"
console_output = true
file_output = false

[[modules]]
module = "qt_rust_demo::pipeline::*"
level = "trace"  # 所有 pipeline 模块显示详细日志

[[modules]]
module = "qt_rust_demo::data_sources::*"
level = "debug"
```

## 通配符匹配规则

支持两种通配符模式：

1. **模块通配符**: `qt_rust_demo::pipeline::*`
   - 匹配 `qt_rust_demo::pipeline::storage_pipeline`
   - 匹配 `qt_rust_demo::pipeline::pipeline_manager`
   - 匹配 `qt_rust_demo::pipeline::storage_queue`

2. **前缀通配符**: `qt_rust_demo::pipeline*`
   - 匹配 `qt_rust_demo::pipeline`
   - 匹配 `qt_rust_demo::pipeline_manager`
   - 匹配 `qt_rust_demo::pipeline_anything`

## 环境变量覆盖

可以使用 `RUST_LOG` 环境变量临时覆盖配置文件：

```bash
# 显示所有 trace 级别日志
RUST_LOG=trace ./qt-rust-demo

# 只显示特定模块的 debug 日志
RUST_LOG=qt_rust_demo::pipeline=debug ./qt-rust-demo

# 复杂配置
RUST_LOG=warn,qt_rust_demo::pipeline=trace,qt_rust_demo::repositories=debug ./qt-rust-demo
```

## 实际使用示例

### 在 storage_pipeline.rs 中使用

```rust
// src/pipeline/storage_pipeline.rs
use tracing::{debug, info, warn, error};

pub struct StoragePipeline {
    // ...
}

impl StoragePipeline {
    pub fn process_data(&self, data: SensorData) {
        debug!("开始处理数据: {:?}", data);
        
        if let Err(e) = self.validate_data(&data) {
            warn!("数据验证失败: {}", e);
            return;
        }
        
        info!("数据处理成功: id={}", data.id);
    }
    
    fn validate_data(&self, data: &SensorData) -> Result<(), String> {
        if data.value < 0.0 {
            error!("数据值为负: {}", data.value);
            return Err("Invalid data".to_string());
        }
        Ok(())
    }
}
```

### 在 sqlite_storage_repository.rs 中使用

```rust
// src/repositories/sqlite_storage_repository.rs
use tracing::{debug, info, error};

impl SqliteStorageRepository {
    pub fn save(&self, data: &SensorData) -> Result<(), Error> {
        debug!("保存数据到数据库: {:?}", data);
        
        match self.execute_insert(data) {
            Ok(_) => {
                info!("数据保存成功: id={}", data.id);
                Ok(())
            }
            Err(e) => {
                error!("数据保存失败: {}", e);
                Err(e)
            }
        }
    }
}
```

## 性能考虑

1. **日志级别检查**: 使用 `tracing` 宏时，如果日志级别不满足，宏内的代码不会执行
2. **格式化开销**: 只有在日志会被输出时才会进行字符串格式化
3. **文件 I/O**: 启用文件输出会有一定性能开销，生产环境建议只记录 `warn` 和 `error`

## 故障排查

### 问题 1: 日志没有输出

检查：
1. 是否调用了 `init_logging` 或 `init_logging_from_file`
2. 配置文件路径是否正确
3. 模块路径是否匹配（使用 `module_path!()` 查看）
4. 日志级别是否设置为 `off`

### 问题 2: 日志输出过多

解决：
1. 提高默认日志级别（如从 `debug` 改为 `info`）
2. 为特定模块设置更高的级别
3. 使用 `off` 关闭不需要的模块

### 问题 3: 无法写入日志文件

检查：
1. 日志文件目录是否存在（需要手动创建 `logs/` 目录）
2. 是否有写入权限
3. 磁盘空间是否充足

## 最佳实践

1. **开发阶段**: 使用 `debug` 或 `trace` 级别，便于调试
2. **测试阶段**: 使用 `info` 级别，记录关键操作
3. **生产环境**: 使用 `warn` 级别，只记录异常情况
4. **关键模块**: 可以单独提高日志级别进行问题排查
5. **日志内容**: 包含足够的上下文信息（ID、状态、参数等）
6. **敏感信息**: 避免记录密码、密钥等敏感数据

## 下一步

1. 根据需要调整 `config/logging.toml` 配置
2. 在关键代码路径添加日志
3. 运行应用程序，观察日志输出
4. 根据实际情况优化日志级别

---

**更新日期**: 2026-03-25
**状态**: ✅ 已实现并可用
