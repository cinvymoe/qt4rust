# 报警管理模块

## 概述

报警管理模块采用策略模式 + 责任链模式，实现了可扩展的多类型报警系统。

## 架构设计

```
AlarmManager (管理器)
    ├── AlarmChecker (检查器接口)
    │   ├── MomentAlarmChecker (力矩报警)
    │   ├── AngleAlarmChecker (角度报警)
    │   ├── LoadOverloadChecker (载荷超限)
    │   └── ... (可扩展)
    ├── AlarmConfig (配置)
    └── DebounceLogic (防抖机制)
```

## 核心组件

### 1. AlarmType (报警类型)

定义报警的来源和级别：

```rust
pub enum AlarmSource {
    Moment,           // 力矩报警
    Angle,            // 角度报警
    MainHookSwitch,   // 主钩勾头开关
    AuxHookSwitch,    // 副钩勾头开关
    LoadOverload,     // 载荷超限
    SensorFault,      // 传感器故障
    SystemError,      // 系统错误
}

pub enum AlarmLevel {
    Warning,   // 预警
    Danger,    // 危险
    Critical,  // 严重
}
```

### 2. AlarmChecker (报警检查器)

策略模式接口，每种报警类型实现一个检查器：

```rust
pub trait AlarmChecker: Send + Sync {
    fn check(&self, data: &ProcessedData) -> AlarmCheckResult;
    fn source(&self) -> AlarmSource;
    fn is_enabled(&self) -> bool;
}
```

### 3. AlarmManager (报警管理器)

责任链模式，管理所有检查器：

```rust
let config = AlarmConfig::default();
let manager = AlarmManager::new(config);

// 检查所有报警
let results = manager.check_alarms(&processed_data);

// 处理报警结果
for result in results {
    if result.triggered {
        println!("报警: {}", result.message);
    }
}
```

## 使用示例

### 基本使用

```rust
use crate::alarm::{AlarmManager, AlarmConfig};

// 1. 创建配置
let config = AlarmConfig::default();

// 2. 创建管理器
let manager = AlarmManager::new(config);

// 3. 检查报警
let results = manager.check_alarms(&processed_data);

// 4. 处理结果
for result in results {
    if result.triggered {
        // 保存报警记录
        repository.save_alarm_record(&result);
    }
}
```

### 自定义配置

```rust
let mut config = AlarmConfig::default();

// 启用角度报警
config.set_alarm_enabled(AlarmSource::Angle, true);

// 修改力矩阈值
config.moment.warning_threshold = 85.0;
config.moment.danger_threshold = 95.0;

// 调整防抖参数
config.debounce.trigger_count = 3;
config.debounce.clear_count = 5;

let manager = AlarmManager::new(config);
```

### 动态更新配置

```rust
// 运行时更新配置
let new_config = load_config_from_file("config/alarm_config.toml")?;
manager.update_config(new_config);
```

## 扩展新报警类型

### 步骤 1: 添加报警来源

```rust
// src/alarm/alarm_type.rs
pub enum AlarmSource {
    // ... 现有类型
    WindSpeed,  // 新增：风速报警
}
```

### 步骤 2: 实现检查器

```rust
// src/alarm/alarm_checker.rs
pub struct WindSpeedChecker {
    max_wind_speed: f64,
    enabled: bool,
}

impl AlarmChecker for WindSpeedChecker {
    fn check(&self, data: &ProcessedData) -> AlarmCheckResult {
        let wind_speed = data.wind_speed; // 假设数据中有风速
        
        if wind_speed > self.max_wind_speed {
            AlarmCheckResult::alarm(
                AlarmType::new(AlarmSource::WindSpeed, AlarmLevel::Danger),
                format!("风速 {:.1} m/s 超过限制", wind_speed),
                wind_speed,
            )
        } else {
            AlarmCheckResult::no_alarm()
        }
    }
    
    fn source(&self) -> AlarmSource {
        AlarmSource::WindSpeed
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
```

### 步骤 3: 添加配置

```rust
// src/alarm/alarm_config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindSpeedConfig {
    pub max_wind_speed: f64,
}

impl Default for WindSpeedConfig {
    fn default() -> Self {
        Self {
            max_wind_speed: 15.0, // 默认 15 m/s
        }
    }
}

// 在 AlarmConfig 中添加
pub struct AlarmConfig {
    // ... 现有字段
    #[serde(default)]
    pub wind_speed: WindSpeedConfig,
}
```

### 步骤 4: 注册到管理器

```rust
// src/alarm/alarm_manager.rs
impl AlarmManager {
    pub fn new(config: AlarmConfig) -> Self {
        // ... 现有代码
        
        // 添加风速报警检查器
        if config.is_alarm_enabled(AlarmSource::WindSpeed) {
            checkers.push(Box::new(WindSpeedChecker::new(
                config.wind_speed.max_wind_speed,
            )));
        }
        
        // ...
    }
}
```

## 集成到现有系统

### 替换 StoragePipeline 中的报警逻辑

```rust
// src/pipeline/storage_pipeline.rs

use crate::alarm::{AlarmManager, AlarmConfig};

pub struct StoragePipeline {
    // ... 现有字段
    alarm_manager: AlarmManager,
}

impl StoragePipeline {
    pub fn new(config: StoragePipelineConfig, ...) -> Self {
        // 加载报警配置
        let alarm_config = AlarmConfig::load_from_file("config/alarm_config.toml")
            .unwrap_or_default();
        
        Self {
            // ... 现有字段
            alarm_manager: AlarmManager::new(alarm_config),
        }
    }
    
    async fn handle_new_data(...) {
        for data in data_list {
            // 使用报警管理器检查报警
            let alarm_results = self.alarm_manager.check_alarms(&data);
            
            // 处理报警结果
            for result in alarm_results {
                if result.triggered {
                    Self::save_alarm_inner(data.clone(), config, repository).await;
                }
            }
            
            // ... 其他逻辑
        }
    }
}
```

## 配置文件

配置文件位于 `config/alarm_config.toml`：

```toml
[moment]
warning_threshold = 90.0
danger_threshold = 100.0

[angle]
min_angle = 0.0
max_angle = 85.0

[debounce]
trigger_count = 5
clear_count = 10
enabled = true

[enabled_alarms]
moment = true
angle = false
load_overload = false
```

## 优势

1. **解耦**: 报警逻辑独立，不影响其他模块
2. **可扩展**: 新增报警类型只需实现 AlarmChecker
3. **可配置**: 所有参数可通过配置文件调整
4. **可测试**: 每个检查器可独立测试
5. **防抖机制**: 避免误报和高频报警
6. **多级报警**: 支持预警、危险、严重三级

## 测试

```rust
#[test]
fn test_alarm_manager() {
    let config = AlarmConfig::default();
    let manager = AlarmManager::new(config);
    
    let sensor_data = SensorData::new(23.0, 10.0, 60.0);
    let processed = ProcessedData::from_sensor_data(sensor_data, 1);
    
    let results = manager.check_alarms(&processed);
    assert!(!results.is_empty());
}
```

## 性能考虑

- 检查器按优先级排序，高优先级先检查
- 使用 RwLock 保证线程安全
- 防抖机制减少不必要的报警记录
- 配置热重载不影响运行中的检查

## 未来扩展

- [ ] 报警优先级排序
- [ ] 报警静音功能
- [ ] 报警历史统计
- [ ] 报警通知（声音、邮件、短信）
- [ ] 报警规则引擎（复杂条件组合）
