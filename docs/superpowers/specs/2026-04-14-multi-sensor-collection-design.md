# 多传感器采集模块解耦设计

## 1. 设计目标

### 1.1 需求概述

将传感器采集模块解耦，实现：
- 支持多种类型传感器（AD采集、Modbus、CAN总线等）
- 统一管理层调度所有传感器
- 多传感器数据汇总到统一数据结构
- 支持轮询和事件驱动双采集模式

### 1.2 设计原则

1. **传感器自治** - 每个传感器管理自己的配置，实现自己的 `read()` 接口
2. **统一抽象** - 通过 `Sensor` trait 统一接口
3. **灵活扩展** - 新增传感器只需实现 `Sensor` trait，无需修改管理层
4. **数据聚合** - 管理层负责汇总多传感器数据到统一结构

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                   SensorManager（统一管理层）                │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ • 传感器注册表                                         │ │
│  │ • 采集调度（轮询/事件）                                 │ │
│  │ • 数据聚合                                             │ │
│  │ • 故障管理                                             │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
              ↑ 遍历调用                    ↓ 统一输出
┌─────────────┬─────────────┬─────────────┬───────────────┐
│  ADSensor   │ ModbusSensor│  CANSensor  │ Future Sensors│
│             │             │             │               │
│ • 自有配置   │ • 自有配置   │ • 自有配置   │ • 自有配置    │
│ • read()    │ • read()    │ • read()    │ • read()     │
│ • 校准      │ • 协议解析   │ • 帧解析    │ • ...        │
└─────────────┴─────────────┴─────────────┴───────────────┘
```

### 2.2 数据流

```
传感器1 ──┐
传感器2 ──┼──→ SensorManager ──→ UnifiedSensorData ──→ 处理管道
传感器3 ──┤    (聚合调度)        (统一数据结构)         (现有)
  ...   ──┘
```

---

## 3. 核心接口设计

### 3.1 Sensor Trait

```rust
// src/sensors/mod.rs

use std::time::SystemTime;
use std::sync::mpsc::{Receiver, Sender};

/// 传感器错误类型
#[derive(Debug, Clone)]
pub enum SensorError {
    NotSupported(String),
    ConnectionFailed(String),
    ReadFailed(String),
    Timeout,
    InvalidData(String),
}

/// 采集模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionMode {
    /// 轮询模式 - 定时主动读取
    Polling,
    /// 事件模式 - 等待信号量/中断触发
    Event,
}

/// 传感器健康状态
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Warning(String),
    Error(String),
    Disconnected,
}

/// 传感器 trait - 所有传感器必须实现此接口
pub trait Sensor: Send + Sync {
    // ========== 标识信息 ==========
    
    /// 传感器唯一ID
    fn id(&self) -> &str;
    
    /// 传感器名称（用于显示）
    fn name(&self) -> &str;
    
    /// 传感器类型
    fn sensor_type(&self) -> &str;
    
    /// 支持的采集模式
    fn supported_modes(&self) -> Vec<CollectionMode>;
    
    // ========== 轮询模式接口 ==========
    
    /// 同步读取数据（轮询模式使用）
    fn read(&mut self) -> Result<SensorData, SensorError>;
    
    // ========== 事件驱动模式接口 ==========
    
    /// 启动异步采集（事件模式使用）
    fn start_async(&mut self) -> Result<(), SensorError>;
    
    /// 停止异步采集
    fn stop_async(&mut self);
    
    /// 获取数据通道（事件模式使用）
    fn data_channel(&self) -> Option<&Receiver<SensorData>>;
    
    // ========== 通用控制接口 ==========
    
    /// 健康检查
    fn health_check(&self) -> HealthStatus;
    
    /// 重置传感器
    fn reset(&mut self) -> Result<(), SensorError>;
    
    /// 获取配置（用于调试/日志）
    fn config_summary(&self) -> String;
}

/// 传感器原始数据
#[derive(Debug, Clone)]
pub struct SensorData {
    /// 采集时间戳
    pub timestamp: SystemTime,
    
    /// 数据点（键值对形式，支持任意传感器）
    pub values: HashMap<String, f64>,
}
```

### 3.2 统一数据结构

```rust
// src/sensors/unified_data.rs

use std::collections::HashMap;
use std::time::SystemTime;

/// 统一传感器数据 - 管理层输出的聚合数据结构
#[derive(Debug, Clone)]
pub struct UnifiedSensorData {
    /// 采集时间戳
    pub timestamp: SystemTime,
    
    /// 采集周期序号
    pub sequence_number: u64,
    
    /// ========== 核心数据（来自主要传感器）==========
    pub core: CoreSensorData,
    
    /// ========== 扩展数据（来自其他传感器）==========
    pub extensions: HashMap<String, SensorValue>,
    
    /// ========== 元数据 ==========
    pub metadata: DataMetadata,
}

/// 核心传感器数据（起重机核心参数）
#[derive(Debug, Clone, Default)]
pub struct CoreSensorData {
    /// 当前载荷（吨）
    pub load: f64,
    
    /// 工作半径（米）
    pub radius: f64,
    
    /// 吊臂角度（度）
    pub angle: f64,
    
    /// 额定载荷（吨）
    pub rated_load: f64,
    
    /// 臂长（米）
    pub boom_length: f64,
}

/// 通用传感器值类型
#[derive(Debug, Clone)]
pub enum SensorValue {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
    Raw(Vec<u8>),
}

/// 数据元数据
#[derive(Debug, Clone)]
pub struct DataMetadata {
    /// 数据完整度（成功采集的传感器数量 / 总传感器数量）
    pub completeness: f64,
    
    /// 各传感器采集状态
    pub sensor_status: HashMap<String, SensorStatus>,
    
    /// 采集耗时（毫秒）
    pub collection_duration_ms: u64,
}

/// 单个传感器状态
#[derive(Debug, Clone)]
pub struct SensorStatus {
    /// 是否成功
    pub success: bool,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 采集时间戳
    pub timestamp: SystemTime,
}
```

---

## 4. 传感器管理器设计

### 4.1 SensorManager

```rust
// src/sensors/manager.rs

/// 传感器管理器配置
#[derive(Debug, Clone)]
pub struct SensorManagerConfig {
    /// 采集模式
    pub mode: CollectionMode,
    
    /// 轮询间隔（毫秒）
    pub polling_interval_ms: u64,
    
    /// 采集超时（毫秒）
    pub timeout_ms: u64,
    
    /// 连续失败阈值（超过此值触发警告）
    pub failure_threshold: u32,
}

impl Default for SensorManagerConfig {
    fn default() -> Self {
        Self {
            mode: CollectionMode::Polling,
            polling_interval_ms: 100,
            timeout_ms: 1000,
            failure_threshold: 5,
        }
    }
}

/// 传感器管理器 - 统一管理层
pub struct SensorManager {
    /// 注册的传感器
    sensors: Vec<Box<dyn Sensor>>,
    
    /// 管理器配置
    config: SensorManagerConfig,
    
    /// 序列号计数器
    sequence_counter: u64,
    
    /// 连续失败计数
    failure_counts: HashMap<String, u32>,
}

impl SensorManager {
    /// 创建新的传感器管理器
    pub fn new(config: SensorManagerConfig) -> Self {
        Self {
            sensors: Vec::new(),
            config,
            sequence_counter: 0,
            failure_counts: HashMap::new(),
        }
    }
    
    /// 注册传感器
    pub fn register_sensor(&mut self, sensor: Box<dyn Sensor>) {
        self.failure_counts.insert(sensor.id().to_string(), 0);
        self.sensors.push(sensor);
    }
    
    /// 注销传感器
    pub fn unregister_sensor(&mut self, sensor_id: &str) -> Option<Box<dyn Sensor>> {
        self.failure_counts.remove(sensor_id);
        self.sensors.iter().position(|s| s.id() == sensor_id)
            .and_then(|idx| self.sensors.remove(idx))
    }
    
    /// 获取下一个序列号
    fn next_sequence(&mut self) -> u64 {
        self.sequence_counter += 1;
        self.sequence_counter
    }
    
    /// 采集并聚合所有传感器数据
    pub fn collect_unified(&mut self) -> UnifiedSensorData {
        let start_time = std::time::Instant::now();
        let timestamp = SystemTime::now();
        let sequence = self.next_sequence();
        
        let mut core = CoreSensorData::default();
        let mut extensions = HashMap::new();
        let mut sensor_status = HashMap::new();
        let mut success_count = 0;
        
        // 遍历所有传感器采集数据
        for sensor in &mut self.sensors {
            let result = sensor.read();
            
            match result {
                Ok(data) => {
                    success_count += 1;
                    self.failure_counts.insert(sensor.id().to_string(), 0);
                    
                    // 根据传感器类型聚合数据
                    self.aggregate_data(sensor.id(), sensor.sensor_type(), data, &mut core, &mut extensions);
                    
                    sensor_status.insert(
                        sensor.id().to_string(),
                        SensorStatus {
                            success: true,
                            error: None,
                            timestamp: SystemTime::now(),
                        },
                    );
                }
                Err(e) => {
                    let count = self.failure_counts.entry(sensor.id().to_string()).or_insert(0);
                    *count += 1;
                    
                    sensor_status.insert(
                        sensor.id().to_string(),
                        SensorStatus {
                            success: false,
                            error: Some(e.to_string()),
                            timestamp: SystemTime::now(),
                        },
                    );
                    
                    // 超过阈值打印警告
                    if *count >= self.config.failure_threshold {
                        tracing::warn!(
                            "传感器 {} 连续失败 {} 次",
                            sensor.id(),
                            count
                        );
                    }
                }
            }
        }
        
        UnifiedSensorData {
            timestamp,
            sequence_number: sequence,
            core,
            extensions,
            metadata: DataMetadata {
                completeness: if self.sensors.is_empty() {
                    1.0
                } else {
                    success_count as f64 / self.sensors.len() as f64
                },
                sensor_status,
                collection_duration_ms: start_time.elapsed().as_millis() as u64,
            },
        }
    }
    
    /// 根据传感器类型聚合数据到核心或扩展字段
    fn aggregate_data(
        &self,
        sensor_id: &str,
        sensor_type: &str,
        data: SensorData,
        core: &mut CoreSensorData,
        extensions: &mut HashMap<String, SensorValue>,
    ) {
        match sensor_type {
            "ad_sensor" => {
                // AD 传感器数据映射到核心字段
                core.load = data.values.get("load").copied().unwrap_or(0.0);
                core.radius = data.values.get("radius").copied().unwrap_or(0.0);
                core.angle = data.values.get("angle").copied().unwrap_or(0.0);
            }
            _ => {
                // 其他传感器数据放入扩展字段
                for (key, value) in data.values {
                    extensions.insert(
                        format!("{}/{}", sensor_id, key),
                        SensorValue::Float(*value),
                    );
                }
            }
        }
    }
    
    /// 获取传感器列表
    pub fn sensors(&self) -> &[Box<dyn Sensor>] {
        &self.sensors
    }
    
    /// 获取传感器数量
    pub fn sensor_count(&self) -> usize {
        self.sensors.len()
    }
    
    /// 健康检查汇总
    pub fn health_summary(&self) -> HashMap<String, HealthStatus> {
        self.sensors
            .iter()
            .map(|s| (s.id().to_string(), s.health_check()))
            .collect()
    }
}
```

---

## 5. 传感器实现示例

### 5.1 AD 传感器

```rust
// src/sensors/ad_sensor.rs

/// AD 传感器实现
pub struct ADSensor {
    id: String,
    name: String,
    config: ADSensorConfig,
}

impl ADSensor {
    pub fn new(id: &str, name: &str, config_path: &str) -> Result<Self, SensorError> {
        Ok(Self {
            id: id.to_string(),
            name: name.to_string(),
            config: ADSensorConfig::load(config_path)?,
        })
    }
}

impl Sensor for ADSensor {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn sensor_type(&self) -> &str {
        "ad_sensor"
    }
    
    fn supported_modes(&self) -> Vec<CollectionMode> {
        vec![CollectionMode::Polling]  // AD 传感器只支持轮询
    }
    
    fn read(&mut self) -> Result<SensorData, SensorError> {
        // 从 AD 硬件读取原始值
        let raw_load = self.read_ad_channel(self.config.load_channel)?;
        let raw_radius = self.read_ad_channel(self.config.radius_channel)?;
        let raw_angle = self.read_ad_channel(self.config.angle_channel)?;
        
        // 应用校准转换
        let load = self.config.calibrate_load(raw_load);
        let radius = self.config.calibrate_radius(raw_radius);
        let angle = self.config.calibrate_angle(raw_angle);
        
        Ok(SensorData {
            timestamp: SystemTime::now(),
            values: [
                ("load".to_string(), load),
                ("radius".to_string(), radius),
                ("angle".to_string(), angle),
            ].into_iter().collect(),
        })
    }
    
    fn start_async(&mut self) -> Result<(), SensorError> {
        Err(SensorError::NotSupported("AD sensor does not support async mode".to_string()))
    }
    
    fn stop_async(&mut self) {
        // 不支持，无操作
    }
    
    fn data_channel(&self) -> Option<&Receiver<SensorData>> {
        None
    }
    
    fn health_check(&self) -> HealthStatus {
        // AD 传感器健康检查
        HealthStatus::Healthy
    }
    
    fn reset(&mut self) -> Result<(), SensorError> {
        // 重置 AD 采集卡
        Ok(())
    }
    
    fn config_summary(&self) -> String {
        format!("AD Sensor {}: channels={:?}", self.id, self.config)
    }
}
```

---

## 6. 使用示例

### 6.1 创建和配置

```rust
// 创建传感器管理器
let config = SensorManagerConfig {
    mode: CollectionMode::Polling,
    polling_interval_ms: 100,
    ..Default::default()
};
let mut manager = SensorManager::new(config);

// 注册传感器（各自管理配置）
manager.register_sensor(Box::new(ADSensor::new(
    "ad_1",
    "主载荷传感器",
    "config/ad_sensor.toml",
)?));

manager.register_sensor(Box::new(ADSensor::new(
    "ad_2", 
    "辅助传感器",
    "config/ad_sensor_2.toml",
)?));

// 后续可扩展注册其他类型传感器
// manager.register_sensor(Box::new(ModbusSensor::new(...))?);
```

### 6.2 数据采集

```rust
// 采集统一数据
let unified_data = manager.collect_unified();

// 访问核心数据
println!("载荷: {:.2}t, 半径: {:.2}m, 角度: {:.1}°",
    unified_data.core.load,
    unified_data.core.radius,
    unified_data.core.angle
);

// 检查数据完整度
if unified_data.metadata.completeness < 1.0 {
    for (sensor_id, status) in &unified_data.metadata.sensor_status {
        if !status.success {
            eprintln!("传感器 {} 失败: {}", sensor_id, status.error.as_ref().unwrap());
        }
    }
}
```

---

## 7. 与现有系统的集成

### 7.1 转换为现有数据结构

```rust
// 转换为现有的 SensorData（保持向后兼容）
impl From<UnifiedSensorData> for crate::models::SensorData {
    fn from(data: UnifiedSensorData) -> Self {
        crate::models::SensorData::new(
            data.core.load,
            data.core.radius,
            data.core.angle,
        )
    }
}
```

### 7.2 输出到处理管道

```rust
// SensorManager 输出直接对接现有处理管道
let unified = manager.collect_unified();
let sensor_data: SensorData = unified.clone().into();

// 写入共享缓冲区
if let Ok(mut buf) = buffer.write() {
    let processed = ProcessedData::from_sensor_data(sensor_data, unified.sequence_number);
    buf.push(processed);
}
```

---

## 8. 文件结构

```
src/sensors/
├── mod.rs                    # 模块入口，Sensors trait 定义
├── unified_data.rs           # 统一数据结构
├── manager.rs                # SensorManager 实现
├── ad_sensor.rs              # AD 传感器实现
├── modbus_sensor.rs          # Modbus 传感器（未来扩展）
├── can_sensor.rs             # CAN 传感器（未来扩展）
└── errors.rs                 # 错误类型定义
```

---

## 9. 实现计划

### Phase 1: 核心接口
- [ ] 定义 `Sensor` trait
- [ ] 定义 `UnifiedSensorData` 结构
- [ ] 实现 `SensorManager`
- [ ] 实现 `ADSensor`

### Phase 2: 扩展支持
- [ ] 添加 Modbus 传感器支持
- [ ] 添加 CAN 传感器支持
- [ ] 添加事件驱动模式

### Phase 3: 优化
- [ ] 添加更多校准算法
- [ ] 添加数据缓存
- [ ] 添加性能监控

---

## 10. 版本信息

**版本**: 1.0  
**日期**: 2026-04-14  
**状态**: 设计完成，待用户批准后实施
