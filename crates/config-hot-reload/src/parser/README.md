# 配置解析模块

配置解析模块提供统一的 TOML 和 CSV 配置文件解析功能。

## 功能特性

- ✅ TOML 配置文件解析（传感器校准、报警阈值、日志、管道配置）
- ✅ CSV 配置文件解析（额定负载表）
- ✅ UTF-8 编码验证
- ✅ 详细的错误信息（包含文件路径和行号）
- ✅ 统一的错误处理

## 模块结构

```
parser/
├── mod.rs              # 模块入口，提供 ConfigParser 统一接口
├── toml_parser.rs      # TOML 解析器
├── csv_parser.rs       # CSV 解析器
└── README.md           # 本文档
```

## 使用示例

### 基本用法

```rust
use config_hot_reload::parser::ConfigParser;
use std::path::Path;

// 解析传感器校准配置
let config = ConfigParser::parse_sensor_calibration(
    Path::new("config/sensor_calibration.toml")
)?;

// 解析报警阈值配置
let thresholds = ConfigParser::parse_alarm_thresholds(
    Path::new("config/alarm_thresholds.toml")
)?;

// 解析日志配置
let log_config = ConfigParser::parse_logging_config(
    Path::new("config/logging.toml")
)?;

// 解析管道配置
let pipeline_config = ConfigParser::parse_pipeline_config(
    Path::new("config/pipeline_config.toml")
)?;

// 解析额定负载表
let load_table = ConfigParser::parse_rated_load_table(
    Path::new("config/rated_load_table.csv")
)?;
```

### 错误处理

```rust
use config_hot_reload::error::HotReloadError;

match ConfigParser::parse_sensor_calibration(path) {
    Ok(config) => {
        println!("配置加载成功");
    }
    Err(HotReloadError::FileRead { path, source }) => {
        eprintln!("文件读取失败: {}, 原因: {}", path.display(), source);
    }
    Err(HotReloadError::EncodingError { path, source }) => {
        eprintln!("编码错误: {}, 原因: {}", path.display(), source);
    }
    Err(HotReloadError::ParseError { path, reason }) => {
        eprintln!("解析失败: {}, 原因: {}", path.display(), reason);
    }
    Err(e) => {
        eprintln!("未知错误: {}", e);
    }
}
```

## TOML 解析器

### 功能

- 解析 TOML 格式的配置文件
- 支持任意实现 `serde::Deserialize` 的类型
- 自动验证 UTF-8 编码
- 提供详细的错误信息（包含行号）

### 支持的配置类型

1. **传感器校准配置** (`SensorCalibration`)
   - 文件: `sensor_calibration.toml`
   - 包含: 重量、角度、半径传感器的校准参数

2. **报警阈值配置** (`AlarmThresholds`)
   - 文件: `alarm_thresholds.toml`
   - 包含: 力矩预警和报警阈值

3. **日志配置** (`LogConfig`)
   - 文件: `logging.toml`
   - 包含: 日志级别、输出方式、模块配置

4. **管道配置** (`PipelineConfig`)
   - 文件: `pipeline_config.toml`
   - 包含: 采集、处理、存储、显示管道配置

### 示例配置文件

**sensor_calibration.toml**:
```toml
[weight]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 50.0
multiplier = 1.0

[angle]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 90.0
multiplier = 1.0

[radius]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 20.0
multiplier = 1.0
```

**alarm_thresholds.toml**:
```toml
[moment]
warning_percentage = 85.0
alarm_percentage = 95.0
```

## CSV 解析器

### 功能

- 解析 CSV 格式的额定负载表
- 支持注释行（以 `#` 开头）
- 解析阈值配置
- 自动验证数据格式和类型
- 提供详细的错误信息（包含行号）

### 文件格式

```csv
# 注释行
moment_warning_threshold,85.0
moment_alarm_threshold,95.0

boom_length_m,working_radius_m,rated_load_ton
10.0,3.0,50.0
10.0,5.0,40.0
10.0,8.0,30.0
```

### 格式说明

1. **注释行**: 以 `#` 开头，会被忽略
2. **阈值配置**: 
   - `moment_warning_threshold,<value>`: 力矩预警阈值
   - `moment_alarm_threshold,<value>`: 力矩报警阈值
3. **表头**: `boom_length_m,working_radius_m,rated_load_ton`
4. **数据行**: `<臂长>,<工作幅度>,<额定载荷>`

### 数据验证

CSV 解析器会自动验证：
- 数据行格式（必须是 3 列）
- 数值类型（必须能解析为 f64）
- 表格非空（至少有一条数据）

## 错误类型

### FileRead

文件读取失败，可能原因：
- 文件不存在
- 文件权限不足
- 磁盘 I/O 错误

### EncodingError

文件编码错误，可能原因：
- 文件不是 UTF-8 编码
- 文件包含无效的 UTF-8 字节序列

### ParseError

配置解析失败，可能原因：
- TOML 语法错误
- CSV 格式错误
- 数据类型不匹配
- 缺少必需字段

## 测试

### 运行单元测试

```bash
cargo test -p config-hot-reload
```

### 运行示例程序

```bash
cargo run --example test_parser -p config-hot-reload
```

## 设计原则

1. **统一接口**: 通过 `ConfigParser` 提供统一的解析接口
2. **详细错误**: 错误信息包含文件路径、行号和具体原因
3. **类型安全**: 利用 Rust 类型系统保证配置正确性
4. **编码验证**: 强制要求 UTF-8 编码，避免编码问题
5. **可测试性**: 每个函数都有完整的单元测试

## 依赖关系

- `toml`: TOML 解析
- `serde`: 序列化/反序列化
- `qt-rust-demo`: 配置类型定义（主项目）

## 未来扩展

- [ ] 支持 Modbus 配置解析
- [ ] 支持 JSON 格式配置
- [ ] 支持配置文件合并
- [ ] 支持配置文件验证（schema）
- [ ] 支持配置文件热重载（在 manager 模块实现）
