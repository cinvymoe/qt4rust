# Config Hot Reload

配置热加载 crate，为起重机数据显示系统提供运行时配置更新能力。

## 功能特性

- **自动监控配置文件变化** - 使用文件系统监控自动检测配置文件修改
- **支持多种格式** - 支持 TOML 和 CSV 配置文件格式
- **配置有效性验证** - 在应用配置前验证配置的有效性和一致性
- **原子性配置更新** - 确保配置更新的原子性，避免部分更新导致的不一致状态
- **观察者模式通知机制** - 通过订阅者模式通知相关组件配置已更新
- **异步非阻塞设计** - 使用 tokio 异步运行时，配置加载不阻塞主线程
- **详细的错误处理** - 提供清晰的错误信息和日志记录
- **内置组件订阅者** - 为所有核心组件提供开箱即用的配置变更订阅者

## 支持的配置文件

- `sensor_calibration.toml` - 传感器校准配置
- `alarm_thresholds.toml` - 报警阈值配置
- `logging.toml` - 日志配置
- `modbus_sensors.toml` - Modbus 传感器配置
- `pipeline_config.toml` - 管道配置
- `rated_load_table.csv` - 额定负载表

## 使用示例

### 基本使用

```rust
use config_hot_reload::prelude::*;
use config_hot_reload::subscribers::SharedConfigRefs;

#[tokio::main]
async fn main() -> Result<(), HotReloadError> {
    // 创建共享配置引用
    let shared_refs = SharedConfigRefs::default();

    // 创建配置管理器
    let mut manager = HotReloadConfigManager::new("config".into())?;

    // 注册内置订阅者
    config_hot_reload::subscribers::register_all_subscribers(&mut manager, &shared_refs).await;

    // 启动热加载服务
    manager.start().await?;

    // 获取当前配置快照
    let snapshot = manager.get_config_snapshot().await;

    Ok(())
}
```

### 实现自定义配置订阅者

```rust
use config_hot_reload::prelude::*;
use async_trait::async_trait;

struct MySubscriber;

#[async_trait]
impl ConfigSubscriber for MySubscriber {
    async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot) {
        match change.file_type {
            ConfigFileType::SensorCalibration => {
                // 使用 snapshot.sensor_calibration
            }
            ConfigFileType::AlarmThresholds => {
                // 使用 snapshot.alarm_thresholds
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "MySubscriber"
    }
}
```

### 手动重载配置

```rust
// 重载所有配置
manager.reload_all().await?;

// 重载指定配置
manager.reload_config(ConfigFileType::SensorCalibration).await?;
```

### 通过共享引用访问当前配置

```rust
let shared_refs = SharedConfigRefs::default();

// 在任何线程安全的地方读取配置
{
    let cal = shared_refs.sensor_calibration.read().unwrap();
    println!("重量传感器 scale_value: {}", cal.weight.scale_value);
}

{
    let pipeline = shared_refs.pipeline_config.read().unwrap();
    println!("采集间隔: {}ms", pipeline.collection.interval_ms);
}
```

## 内置订阅者

| 订阅者 | 配置类型 | 应用时机 |
|--------|---------|---------|
| `PipelineConfigSubscriber` | `Pipeline` | 下一采集/存储周期 |
| `DataProcessingSubscriber` | `SensorCalibration`, `RatedLoadTable` | 下一次数据转换/力矩计算 |
| `AlarmDetectionSubscriber` | `AlarmThresholds` | 立即生效 |
| `LoggingConfigSubscriber` | `Logging` | 立即生效 |
| `SensorDataSourceSubscriber` | `ModbusSensors` | 下次采集时重连 |

## 架构设计

- **FileWatcher** - 文件监控器，监控配置文件的变化
- **ConfigParser** - 配置解析器，解析 TOML 和 CSV 格式的配置文件
- **ConfigValidator** - 配置验证器，验证配置的有效性和一致性
- **HotReloadConfigManager** - 热加载配置管理器，协调配置热加载流程
- **ConfigSubscriber** - 配置订阅者 trait，定义配置变更通知接口
- **SharedConfigRefs** - 共享配置引用，用于组件和订阅者之间共享配置数据
- **内置订阅者** - 为管道、数据处理、报警、日志、传感器组件提供配置变更处理

## 配置验证规则

- **传感器校准**: `scale_ad - zero_ad != 0`（分母不能为零）
- **报警阈值**: `alarm_percentage >= warning_percentage`，百分比在 0-200 范围内
- **额定负载表**: 半径值升序排列，非空
- **管道配置**: 间隔时间 > 0，缓冲区大小 > 0
- **Modbus 配置**: 端口号在 1-65535 范围内

## 错误处理

- **文件系统错误** - 文件不存在、权限不足、文件被删除或重命名
- **解析错误** - TOML/CSV 语法错误、编码错误
- **验证错误** - 配置参数超出有效范围、配置参数逻辑不一致
- **并发错误** - 锁获取失败
- **订阅者通知错误** - 订阅者处理异常（超时 50ms）

## 性能要求

- 配置文件读取和解析: < 500ms
- 配置验证: < 200ms
- 配置更新和通知: < 100ms
- 配置重载不阻塞数据采集和处理

## 测试

```bash
cargo test -p config-hot-reload
```

## 依赖项

- `notify` - 跨平台文件系统事件监控
- `tokio` - 异步运行时
- `async-trait` - 异步 trait 支持
- `thiserror` - 自定义错误类型
- `tracing` - 结构化日志
- `toml` - TOML 解析
- `csv` - CSV 解析
- `serde` - 序列化/反序列化
- `qt-rust-demo` - 主项目（配置类型定义）

## 许可证

与主项目相同