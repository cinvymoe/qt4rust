# 管道数据结构详解

## 概述

本文档详细说明起重机监控系统中各个管道的数据结构、转换流程和示例数据。

## 1. 数据流总览

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              数据流转换链                                  │
└──────────────────────────────────────────────────────────────────────────┘

传感器原始数据                滤波后数据                  处理后数据
    ↓                            ↓                            ↓
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│ SensorData   │  ────→  │ SensorData   │  ────→  │ ProcessedData│
│              │  滤波    │ (滤波后)     │  AD转换 │              │
│ AD1: f64     │         │ AD1: f64     │         │ Load: 吨     │
│ AD2: f64     │         │ AD2: f64     │         │ Radius: 米   │
│ AD3: f64     │         │ AD3: f64     │         │ Angle: 度    │
└──────────────┘         └──────────────┘         │ Moment%: %   │
                                                  │ is_danger    │
                                                  └──────────────┘
                                                        │
                    ┌───────────────────────────────────┴──────┐
                    ↓                                          ↓
          ┌──────────────────┐                      ┌──────────────────┐
          │ SharedBuffer     │                      │ StoragePipeline  │
          │ (显示用)         │                      │ (存储用)         │
          │ latest: Data     │                      │ SQLite DB        │
          │ history: Vec     │                      │ runtime_data表   │
          └──────────────────┘                      │ alarm_records表  │
                                                    └──────────────────┘
```

---

## 2. 采集管道数据

### 2.1 SensorData - 原始传感器数据

**定义位置**: `src/models/sensor_data.rs`

```rust
pub struct SensorData {
    /// AD1 - 载荷传感器原始值
    pub ad1_load: f64,
    
    /// AD2 - 工作半径传感器原始值（米）
    pub ad2_radius: f64,
    
    /// AD3 - 吊臂角度传感器原始值（度）
    pub ad3_angle: f64,
}
```

**数据来源**:
- 模拟传感器 (SimulatedSensor)
- Modbus TCP 传感器 (ModbusTcpSensor)
- 串口传感器 (SerialSensor)

**数据范围**:
- AD1: 0.0 ~ 4095.0 (12位ADC)
- AD2: 0.0 ~ 4095.0 (12位ADC)
- AD3: 0.0 ~ 4095.0 (12位ADC)

**示例数据**:

| 来源 | ad1_load | ad2_radius | ad3_angle | 说明 |
|------|----------|------------|-----------|------|
| 模拟传感器 | 2047.5 | 2047.5 | 2730.0 | 中等载荷，中等半径，60度角 |
| Modbus TCP | 1023.75 | 3071.25 | 1365.0 | 低载荷，大半径，30度角 |
| 串口传感器 | 3071.25 | 1023.75 | 3412.5 | 高载荷，小半径，75度角 |

**数据流程**:

```
1. 数据采集 (每100ms)
   ├─ CraneDataRepository.get_latest_sensor_data()
   ├─ 使用 spawn_blocking 执行阻塞 I/O
   └─ 返回 SensorData

2. 数据验证
   ├─ ad1_load >= 0.0
   ├─ ad2_radius >= 0.0
   └─ 0.0 <= ad3_angle <= 90.0

3. 数据分发
   ├─ 写入 FilterBuffer (多速率模式)
   │  └─ Arc<Mutex<FilterBuffer>>
   ├─ 写入 SharedSensorBuffer (校准界面)
   │  └─ Arc<RwLock<SensorDataBuffer>>
   └─ 或直接处理 (遗留模式)
      └─ ProcessedData.from_sensor_data()
```

---

### 2.2 FilterBuffer - 滤波缓冲区数据

**定义位置**: `src/pipeline/filter_buffer.rs`

```rust
pub struct FilterBuffer {
    raw_data: VecDeque<SensorData>,
    config: FilterBufferConfig,
}

pub struct FilterBufferConfig {
    pub filter_type: FilterType,    // None, Mean, Median
    pub window_size: usize,          // 滑动窗口大小
}

pub enum FilterType {
    None,    // 不滤波
    Mean,    // 均值滤波
    Median,  // 中值滤波
}
```

**滤波算法**:

#### 均值滤波 (Mean)
```rust
fn mean_filter(data: &[SensorData]) -> SensorData {
    let count = data.len() as f64;
    let (a, r, g) = data.iter().fold((0.0, 0.0, 0.0), |(a, r, g), d| {
        (a + d.ad1_load, r + d.ad2_radius, g + d.ad3_angle)
    });
    SensorData::new(a / count, r / count, g / count)
}
```

**示例**:
```
窗口大小 = 5
最近5个样本: [10, 20, 30, 40, 50]
均值滤波结果: (10+20+30+40+50)/5 = 30
```

#### 中值滤波 (Median)
```rust
fn median_filter(data: &[SensorData]) -> SensorData {
    let mut ad1: Vec<f64> = data.iter().map(|d| d.ad1_load).collect();
    ad1.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = data.len() / 2;
    SensorData::new(ad1[mid], ad2[mid], ad3[mid])
}
```

**示例**:
```
窗口大小 = 5
最近5个样本: [10, 30, 20, 50, 40]
排序后: [10, 20, 30, 40, 50]
中值滤波结果: 30 (中间值)
```

**滤波效果对比**:

| 原始数据序列 | 无滤波 | 均值滤波 | 中值滤波 |
|-------------|--------|----------|----------|
| [10,20,30,40,50] | 50 | 30 | 30 |
| [10,100,20,50,40] | 40 | 44 | 40 |
| [10,30,20,50,40] | 40 | 30 | 30 |

**数据流程**:

```
采集管道 (100ms)
    ↓ SensorData
FilterBuffer.push(data)
    ↓ 添加到队列
队列长度检查
    ├─ < window_size * 2: 正常
    └─ >= window_size * 2: 移除最旧数据
    ↓
计算管道 (100ms)
    ↓ FilterBuffer.get_filtered()
滤波算法选择
    ├─ None: 返回最新数据
    ├─ Mean: 计算窗口均值
    └─ Median: 计算窗口中值
    ↓
返回滤波后的 SensorData
```

---

## 3. 计算管道数据

### 3.1 ProcessedData - 处理后的数据

**定义位置**: `src/models/processed_data.rs`

```rust
pub struct ProcessedData {
    /// 当前载荷（吨）
    pub current_load: f64,

    /// 额定载荷（吨，从载荷表查询得到）
    pub rated_load: f64,

    /// 工作半径（米）
    pub working_radius: f64,

    /// 吊臂角度（度）
    pub boom_angle: f64,

    /// 臂长（米）
    pub boom_length: f64,

    /// 力矩百分比
    pub moment_percentage: f64,

    /// 是否预警（达到预警阈值，>=90%）
    pub is_warning: bool,

    /// 是否危险（达到报警阈值，>=100%）
    pub is_danger: bool,

    /// 验证错误
    pub validation_error: Option<String>,

    /// 时间戳
    pub timestamp: SystemTime,

    /// 序列号
    pub sequence_number: u64,

    /// 当前活动的报警来源列表
    pub alarm_sources: Vec<AlarmSource>,

    /// 报警消息列表
    pub alarm_messages: Vec<String>,
}
```

#### AD转换流程

**步骤1: AD值转换**

```rust
// ad1 -> current_load (载荷)
let current_load = sensor_calibration.convert_weight_ad_to_value(raw_data.ad1_load);

// ad2 -> boom_length (臂长)
let boom_length = sensor_calibration.convert_radius_ad_to_value(raw_data.ad2_radius);

// ad3 -> boom_angle (臂角)
let boom_angle = sensor_calibration.convert_angle_ad_to_value(raw_data.ad3_angle);
```

**转换公式**:
```
物理值 = (AD值 - zero_ad) / (scale_ad - zero_ad) × (scale_value - zero_value) + zero_value
```

**示例**:
```rust
// 默认配置
weight: zero_ad=0, zero_value=0, scale_ad=4095, scale_value=50吨

// AD值 2047.5 的转换
current_load = (2047.5 - 0) / (4095 - 0) × (50 - 0) + 0
             = 0.5 × 50
             = 25吨
```

**步骤2: 计算工作半径**

```rust
fn calculate_working_radius(boom_length: f64, boom_angle_degrees: f64) -> f64 {
    let angle_rad = boom_angle_degrees.to_radians();
    let cos_angle = angle_rad.cos();
    let effective_cos = cos_angle.max(0.0);  // 钳位到0
    boom_length * effective_cos
}
```

**公式**:
```
working_radius = boom_length × cos(boom_angle)
```

**示例**:
```
臂长 = 10米

角度 0°  (水平): radius = 10 × cos(0°)  = 10 × 1.0   = 10.0米
角度 30°:         radius = 10 × cos(30°) = 10 × 0.866 = 8.66米
角度 45°:         radius = 10 × cos(45°) = 10 × 0.707 = 7.07米
角度 60°:         radius = 10 × cos(60°) = 10 × 0.5   = 5.0米
角度 90° (垂直):  radius = 10 × cos(90°) = 10 × 0.0   = 0.0米
```

**步骤3: 查询额定载荷**

```rust
let rated_load = rated_load_table.get_rated_load(boom_length, working_radius);
```

从 `rated_load_table.csv` 查询对应的额定载荷值。

**示例表格**:
```
臂长(m), 幅度(m), 额定载荷(t)
10.0,    5.0,     40.0
10.0,    7.0,     30.0
10.0,    10.0,    25.0
15.0,    8.0,     35.0
```

**步骤4: 计算力矩百分比**

```rust
fn calculate_moment_percentage_with_load(
    current_load: f64,
    working_radius: f64,
    rated_load: f64,
) -> f64 {
    let current_moment = current_load * working_radius;
    let rated_moment = rated_load * working_radius;
    
    if rated_moment > 0.0 {
        ((current_moment / rated_moment) * 100.0).min(100.0)
    } else {
        0.0
    }
}
```

**公式**:
```
力矩 = 载荷 × 工作半径
力矩百分比 = (当前力矩 / 额定力矩) × 100%
```

**示例**:
```
当前载荷: 20吨
工作半径: 10米
额定载荷: 25吨

当前力矩 = 20 × 10 = 200 吨·米
额定力矩 = 25 × 10 = 250 吨·米
力矩百分比 = (200 / 250) × 100% = 80%
```

#### 完整示例

**输入**: SensorData { ad1_load: 2047.5, ad2_radius: 2047.5, ad3_angle: 2730.0 }

**转换过程**:

```
1. AD转换 (默认标定参数)
   ad1=2047.5 → current_load = 25吨
   ad2=2047.5 → boom_length = 10米
   ad3=2730.0 → boom_angle = 60度

2. 计算工作半径
   working_radius = 10 × cos(60°) = 10 × 0.5 = 5.0米

3. 查询额定载荷
   rated_load = 查表(10米, 5米) = 40吨

4. 计算力矩百分比
   current_moment = 25 × 5 = 125 吨·米
   rated_moment = 40 × 5 = 200 吨·米
   moment_percentage = (125 / 200) × 100% = 62.5%

5. 判断报警状态
   is_warning = 62.5% >= 90% = false
   is_danger = 62.5% >= 100% = false
```

**输出**: ProcessedData
```json
{
  "current_load": 25.0,
  "rated_load": 40.0,
  "working_radius": 5.0,
  "boom_angle": 60.0,
  "boom_length": 10.0,
  "moment_percentage": 62.5,
  "is_warning": false,
  "is_danger": false,
  "validation_error": null,
  "timestamp": "2026-04-14T15:30:00Z",
  "sequence_number": 1,
  "alarm_sources": [],
  "alarm_messages": []
}
```

---

## 4. 存储管道数据

### 4.1 运行数据表 (runtime_data)

**表结构**:
```sql
CREATE TABLE runtime_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER UNIQUE NOT NULL,
    timestamp TEXT NOT NULL,
    current_load REAL NOT NULL,
    rated_load REAL NOT NULL,
    working_radius REAL NOT NULL,
    boom_angle REAL NOT NULL,
    boom_length REAL NOT NULL,
    moment_percentage REAL NOT NULL,
    is_warning INTEGER NOT NULL,
    is_danger INTEGER NOT NULL,
    validation_error TEXT
);
```

**字段说明**:

| 字段 | 类型 | 说明 | 示例值 |
|------|------|------|--------|
| id | INTEGER | 自增主键 | 1, 2, 3... |
| sequence_number | INTEGER | 序列号（唯一） | 1, 2, 3... |
| timestamp | TEXT | ISO 8601 时间戳 | "2026-04-14T15:30:00Z" |
| current_load | REAL | 当前载荷（吨） | 25.0 |
| rated_load | REAL | 额定载荷（吨） | 40.0 |
| working_radius | REAL | 工作半径（米） | 5.0 |
| boom_angle | REAL | 吊臂角度（度） | 60.0 |
| boom_length | REAL | 臂长（米） | 10.0 |
| moment_percentage | REAL | 力矩百分比 | 62.5 |
| is_warning | INTEGER | 是否预警 | 0 或 1 |
| is_danger | INTEGER | 是否危险 | 0 或 1 |
| validation_error | TEXT | 验证错误信息 | null 或 "力矩报警..." |

**存储策略**:

1. **批量存储**
   - 批量大小: 10条记录
   - 刷新间隔: 5秒
   - 触发条件: 批量满 或 定时器到期

2. **数据清理**
   - 最大记录数: max_records (配置)
   - 清理阈值: purge_threshold (默认 max_records × 1.1)
   - 自动清理旧记录

**示例数据**:
```sql
INSERT INTO runtime_data VALUES (
    1,                          -- id
    100,                        -- sequence_number
    '2026-04-14T15:30:00Z',    -- timestamp
    25.0,                       -- current_load
    40.0,                       -- rated_load
    5.0,                        -- working_radius
    60.0,                       -- boom_angle
    10.0,                       -- boom_length
    62.5,                       -- moment_percentage
    0,                          -- is_warning
    0,                          -- is_danger
    NULL                        -- validation_error
);
```

### 4.2 报警记录表 (alarm_records)

**表结构**:
```sql
CREATE TABLE alarm_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,
    alarm_type TEXT NOT NULL,
    alarm_level TEXT NOT NULL,
    current_load REAL NOT NULL,
    moment_percentage REAL NOT NULL,
    working_radius REAL NOT NULL,
    boom_angle REAL NOT NULL,
    message TEXT
);
```

**字段说明**:

| 字段 | 类型 | 说明 | 示例值 |
|------|------|------|--------|
| id | INTEGER | 自增主键 | 1, 2, 3... |
| timestamp | TEXT | 报警时间戳 | "2026-04-14T15:30:05Z" |
| sequence_number | INTEGER | 触发报警的序列号 | 105 |
| alarm_type | TEXT | 报警类型 | "moment", "angle" |
| alarm_level | TEXT | 报警级别 | "warning", "danger" |
| current_load | REAL | 报警时载荷 | 30.0 |
| moment_percentage | REAL | 报警时力矩百分比 | 95.0 |
| working_radius | REAL | 报警时工作半径 | 8.0 |
| boom_angle | REAL | 报警时吊臂角度 | 45.0 |
| message | TEXT | 报警消息 | "力矩预警: 95.0% >= 90.0%" |

**报警类型**:

| 类型 | 说明 | 触发条件 |
|------|------|---------|
| moment | 力矩报警 | moment_percentage >= 90% (预警) 或 >= 100% (危险) |
| angle | 角度报警 | angle < min_angle 或 angle > max_angle |
| main_hook_switch | 主钩勾头开关报警 | 根据配置模式触发 |
| aux_hook_switch | 副钩勾头开关报警 | 根据配置模式触发 |
| load_overload | 载荷超限报警 | current_load > rated_load |
| sensor_fault | 传感器故障报警 | 传感器数据异常 |
| system_error | 系统错误报警 | 系统级错误 |

**报警防抖**:

```rust
// 触发报警: 连续 N 次危险才触发
let current_danger_count = danger_count.fetch_add(1, Ordering::Relaxed) + 1;
if current_danger_count >= config.alarm_debounce_count {
    // 触发新报警
}

// 解除报警: 连续 M 次安全才解除
let current_safe_count = safe_count.fetch_add(1, Ordering::Relaxed) + 1;
if current_safe_count >= config.alarm_clear_debounce_count {
    // 解除报警
}
```

**默认防抖参数**:
- 触发报警: 连续 5 次危险 (500ms @ 100ms 采集周期)
- 解除报警: 连续 10 次安全 (1s @ 100ms 采集周期)

**示例数据**:
```sql
INSERT INTO alarm_records VALUES (
    1,                          -- id
    '2026-04-14T15:30:05Z',    -- timestamp
    105,                        -- sequence_number
    'moment',                   -- alarm_type
    'warning',                  -- alarm_level
    30.0,                       -- current_load
    95.0,                       -- moment_percentage
    8.0,                        -- working_radius
    45.0,                       -- boom_angle
    '力矩预警: 95.0% >= 90.0%'  -- message
);
```

### 4.3 传感器原始数据表 (sensor_data)

**表结构**:
```sql
CREATE TABLE sensor_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    sequence_number INTEGER UNIQUE NOT NULL,
    ad1_load REAL NOT NULL,
    ad2_radius REAL NOT NULL,
    ad3_angle REAL NOT NULL
);
```

**字段说明**:

| 字段 | 类型 | 说明 | 示例值 |
|------|------|------|--------|
| id | INTEGER | 自增主键 | 1, 2, 3... |
| timestamp | TEXT | 时间戳 | "2026-04-14T15:30:00Z" |
| sequence_number | INTEGER | 序列号（唯一） | 1, 2, 3... |
| ad1_load | REAL | 载荷AD值 | 2047.5 |
| ad2_radius | REAL | 半径AD值 | 2047.5 |
| ad3_angle | REAL | 角度AD值 | 2730.0 |

**用途**:
- 保存原始传感器数据
- 用于后续分析和校准
- 故障诊断和追溯

---

## 5. 显示管道数据

### 5.1 SharedBuffer - 共享缓冲区

**定义位置**: `src/pipeline/shared_buffer.rs`

```rust
pub struct ProcessedDataBuffer {
    /// 最新数据
    latest: Option<ProcessedData>,
    
    /// 历史数据队列
    history: VecDeque<ProcessedData>,
    
    /// 最大历史容量
    max_history_size: usize,
    
    /// 统计信息
    stats: BufferStats,
}

pub struct BufferStats {
    /// 总采集次数
    pub total_collections: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub error_count: u64,
    /// 最后更新时间
    pub last_update_time: Option<SystemTime>,
}

pub type SharedBuffer = Arc<RwLock<ProcessedDataBuffer>>;
```

**内存管理**:

```rust
// 最大内存限制: 10MB
const MAX_MEMORY: usize = 10 * 1024 * 1024;

pub fn push(&mut self, data: ProcessedData) {
    self.latest = Some(data.clone());
    
    // 限制历史大小（FIFO）
    if self.history.len() >= self.max_history_size {
        self.history.pop_front();
    }
    self.history.push_back(data);
    
    // 检查内存使用
    while self.estimated_memory_usage() > MAX_MEMORY {
        self.history.pop_front();
    }
    
    // 更新统计
    self.stats.total_collections += 1;
    self.stats.success_count += 1;
    self.stats.last_update_time = Some(SystemTime::now());
}
```

**示例数据**:

```
SharedBuffer {
    latest: ProcessedData {
        current_load: 25.0,
        moment_percentage: 62.5,
        ...
    },
    history: VecDeque [
        ProcessedData { seq: 1, moment_percentage: 60.0 },
        ProcessedData { seq: 2, moment_percentage: 61.5 },
        ProcessedData { seq: 3, moment_percentage: 62.0 },
        ProcessedData { seq: 4, moment_percentage: 62.5 },
    ],
    max_history_size: 1000,
    stats: BufferStats {
        total_collections: 1000,
        success_count: 998,
        error_count: 2,
        last_update_time: Some("2026-04-14T15:30:00Z"),
    }
}
```

### 5.2 DisplayPipeline - 显示管道

**定义位置**: `src/pipeline/display_pipeline.rs`

```rust
pub struct DisplayPipeline {
    config: DisplayPipelineConfig,
    buffer: SharedBuffer,
    /// 内部显示缓冲区
    display_buffer: VecDeque<ProcessedData>,
    /// 上次更新时间
    last_update: Option<Instant>,
    /// 是否运行中
    running: bool,
}
```

**工作原理**:

```rust
// 主线程定期调用 (每100ms)
pub fn tick(&mut self) -> bool {
    // 检查更新间隔
    if now.duration_since(last) < self.config.interval {
        return false;
    }
    
    // 从共享缓冲区读取最新数据
    if let Ok(buf) = self.buffer.read() {
        if let Some(latest) = buf.get_latest() {
            // 添加到显示缓冲区
            self.display_buffer.push_back(latest);
            
            // 限制缓冲区大小
            if self.display_buffer.len() >= self.config.pipeline_size {
                self.display_buffer.pop_front();
            }
            
            return true;  // 需要刷新UI
        }
    }
    
    false
}
```

**数据流向**:

```
计算管道
    ↓ ProcessedData
SharedBuffer (Arc<RwLock>)
    ↓ 主线程读取
DisplayPipeline.tick()
    ↓ 获取最新数据
display_buffer (VecDeque)
    ↓ Qt信号槽
ViewModel 更新
    ↓ 数据绑定
QML UI 自动刷新
```

---

## 6. 事件通道数据

### 6.1 StorageEventChannel - 存储事件通道

**定义位置**: `src/pipeline/event_channel.rs`

```rust
pub struct StorageEventSender {
    data_tx: mpsc::Sender<Vec<ProcessedData>>,      // 数据通道
    shutdown_tx: watch::Sender<bool>,                // 关机信号
}

pub struct StorageEventReceiver {
    data_rx: Arc<Mutex<mpsc::Receiver<Vec<ProcessedData>>>>,
    shutdown_rx: watch::Receiver<bool>,
}
```

**事件类型**:

```rust
pub enum StorageEvent {
    /// 新数据到达
    NewData(Vec<ProcessedData>),
    
    /// 报警触发
    Alarm(ProcessedData),
    
    /// 报警解除
    AlarmCleared,
    
    /// 关机请求
    Shutdown,
}
```

**数据流**:

```
计算管道
    ↓ Vec<ProcessedData>
StorageEventSender.try_send_data()
    ↓ mpsc::channel
StorageEventReceiver.recv()
    ↓ tokio::select!
存储管道事件处理
    ├─ 新数据到达
    │  ├─ 批量累积
    │  ├─ 报警防抖
    │  └─ 写入 SQLite
    ├─ 定时刷新
    │  └─ 批量存储
    └─ 关机信号
       └─ 刷新剩余数据并退出
```

### 6.2 SensorDataEventChannel - 传感器数据事件通道

**定义位置**: `src/pipeline/sensor_data_event_channel.rs`

```rust
pub struct SensorDataEventSender {
    data_tx: mpsc::Sender<Vec<SensorData>>,
    shutdown_tx: watch::Sender<bool>,
}

pub struct SensorDataEventReceiver {
    data_rx: Arc<Mutex<mpsc::Receiver<Vec<SensorData>>>>,
    shutdown_rx: watch::Receiver<bool>,
}
```

**用途**:
- 传输原始传感器数据
- 供传感器存储管道使用
- 支持数据分析和校准

---

## 7. 配置数据

### 7.1 SensorCalibration - 传感器标定参数

**定义位置**: `src/models/sensor_calibration.rs`

```rust
pub struct SensorCalibration {
    /// 重量传感器标定参数
    pub weight: SensorCalibrationParams,
    /// 角度传感器标定参数
    pub angle: SensorCalibrationParams,
    /// 半径传感器标定参数
    pub radius: SensorCalibrationParams,
}

pub struct SensorCalibrationParams {
    /// 零点 AD 值
    pub zero_ad: f64,
    /// 零点物理值
    pub zero_value: f64,
    /// 放大 AD 值（满量程）
    pub scale_ad: f64,
    /// 放大物理值
    pub scale_value: f64,
    /// 标定倍率
    pub multiplier: f64,
    /// 实际倍率
    pub actual_multiplier: f64,
}
```

**配置示例** (sensor_calibration.toml):

```toml
[weight]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 50.0    # 50吨
multiplier = 1.0
actual_multiplier = 1.0

[angle]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 90.0    # 90度
multiplier = 1.0
actual_multiplier = 1.0

[radius]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 20.0    # 20米
multiplier = 1.0
actual_multiplier = 1.0
```

### 7.2 AlarmThresholds - 报警阈值配置

**定义位置**: `src/models/sensor_calibration.rs`

```rust
pub struct AlarmThresholds {
    /// 力矩报警阈值
    pub moment: MomentThresholds,
    /// 角度报警阈值
    pub angle: AngleThresholds,
    /// 主钩勾头开关报警
    pub main_hook_switch: HookSwitchThresholds,
    /// 副钩勾头开关报警
    pub aux_hook_switch: HookSwitchThresholds,
}

pub struct MomentThresholds {
    /// 预警百分比（%）
    pub warning_percentage: f64,
    /// 报警百分比（%）
    pub alarm_percentage: f64,
}

pub struct AngleThresholds {
    /// 最小角度报警（度）
    pub min_angle: f64,
    /// 最大角度报警（度）
    pub max_angle: f64,
}
```

**配置示例** (alarm_thresholds.toml):

```toml
[moment]
warning_percentage = 90.0    # 预警: >= 90%
alarm_percentage = 100.0     # 报警: >= 100%

[angle]
min_angle = 0.0              # 最小角度: 0度
max_angle = 85.0             # 最大角度: 85度

[main_hook_switch]
mode = "none"                # none, normally_open, normally_closed

[aux_hook_switch]
mode = "none"
```

### 7.3 RatedLoadTable - 额定载荷表

**定义位置**: `src/models/rated_load_table.rs`

**配置示例** (rated_load_table.csv):

```csv
boom_length,working_radius,rated_load
10.0,5.0,40.0
10.0,7.0,30.0
10.0,10.0,25.0
15.0,8.0,35.0
15.0,10.0,30.0
15.0,15.0,20.0
20.0,10.0,28.0
20.0,15.0,22.0
20.0,20.0,15.0
```

**查询逻辑**:
```rust
pub fn get_rated_load(&self, boom_length: f64, working_radius: f64) -> f64 {
    // 1. 查找精确匹配
    // 2. 如果没有，使用双线性插值
    // 3. 返回额定载荷值
}
```

---

## 8. 数据转换示例

### 完整数据流示例

**场景**: 中等载荷，中等半径，60度角

**步骤1: 原始传感器数据**
```json
SensorData {
  ad1_load: 2047.5,
  ad2_radius: 2047.5,
  ad3_angle: 2730.0
}
```

**步骤2: 滤波** (Mean, window_size=5)
```json
FilterBuffer {
  raw_data: [
    SensorData { ad1: 2000, ad2: 2100, ad3: 2700 },
    SensorData { ad1: 2100, ad2: 2000, ad3: 2800 },
    SensorData { ad1: 2047, ad2: 2047, ad3: 2730 },
    SensorData { ad1: 2050, ad2: 2050, ad3: 2720 },
    SensorData { ad1: 2040, ad2: 2040, ad3: 2740 }
  ]
}
// 滤波后
SensorData {
  ad1_load: 2047.4,  // 均值
  ad2_radius: 2047.4,
  ad3_angle: 2730.0
}
```

**步骤3: AD转换**
```json
ProcessedData {
  current_load: 25.0,     // 2047.5 → 25吨
  boom_length: 10.0,      // 2047.5 → 10米
  boom_angle: 60.0,       // 2730.0 → 60度
  working_radius: 5.0,    // 10 × cos(60°) = 5米
  rated_load: 40.0,       // 查表(10米, 5米) = 40吨
  moment_percentage: 62.5, // (25×5) / (40×5) = 62.5%
  is_warning: false,      // 62.5% < 90%
  is_danger: false,       // 62.5% < 100%
  sequence_number: 1,
  timestamp: "2026-04-14T15:30:00Z"
}
```

**步骤4: 存储到 SQLite**
```sql
-- runtime_data 表
INSERT INTO runtime_data VALUES (
  1, 1, '2026-04-14T15:30:00Z', 25.0, 40.0, 5.0, 
  60.0, 10.0, 62.5, 0, 0, NULL
);
```

**步骤5: 显示到 UI**
```
SharedBuffer {
  latest: ProcessedData { ... }
}

↓ 主线程读取

DisplayPipeline {
  display_buffer: [ ProcessedData { ... } ]
}

↓ Qt信号槽

ViewModel {
  currentLoad: 25.0吨
  momentPercentage: 62.5%
  ...
}

↓ QML数据绑定

UI 显示:
  - 当前载荷: 25.0 吨
  - 工作半径: 5.0 米
  - 吊臂角度: 60.0 度
  - 力矩百分比: 62.5%
  - 状态: 正常 (绿色)
```

---

## 9. 数据大小估算

### 单条数据大小

| 数据类型 | 字段数 | 估算大小 | 说明 |
|---------|--------|----------|------|
| SensorData | 3个 f64 | 24 字节 | AD原始值 |
| ProcessedData | 13个字段 | ~200 字节 | 包含时间戳、字符串等 |
| FilterBuffer (window=10) | 10个 SensorData | 240 字节 | 滤波窗口 |

### 缓冲区大小

| 缓冲区类型 | 容量 | 估算大小 | 说明 |
|-----------|------|----------|------|
| FilterBuffer | 10-20条 | ~480 字节 | 滤波缓冲区 |
| SharedBuffer | 1000条 | ~200 KB | 显示缓冲区 |
| DisplayPipeline | 10-50条 | ~10 KB | 主线程缓冲区 |
| StoragePipeline 批量 | 10-100条 | ~20 KB | 存储批量 |

### 数据库大小

| 表 | 记录数 | 估算大小 | 说明 |
|----|--------|----------|------|
| runtime_data | 10,000 | ~2 MB | 运行数据 |
| alarm_records | 1,000 | ~200 KB | 报警记录 |
| sensor_data | 10,000 | ~240 KB | 原始数据 |

---

## 10. 总结

### 数据流转路径

```
┌────────────────────────────────────────────────────────────────────┐
│                        数据流转完整路径                              │
└────────────────────────────────────────────────────────────────────┘

传感器
  ↓ SensorData (24 bytes)
  ├─ AD1: 载荷原始值
  ├─ AD2: 半径原始值
  └─ AD3: 角度原始值

采集管道 (100ms)
  ↓ 验证、写入

FilterBuffer
  ↓ SensorData (滤波后, 24 bytes)
  ├─ 均值滤波 / 中值滤波
  └─ 窗口大小: 10

计算管道 (100ms)
  ↓ AD转换、计算

ProcessedData (~200 bytes)
  ├─ current_load: 吨
  ├─ working_radius: 米
  ├─ boom_angle: 度
  ├─ moment_percentage: %
  ├─ is_warning / is_danger
  └─ alarm_sources / alarm_messages

存储管道 (事件驱动)
  ├─→ runtime_data 表 (SQLite)
  ├─→ alarm_records 表 (SQLite)
  └─→ sensor_data 表 (SQLite)

显示管道 (主线程, 100ms)
  ├─→ SharedBuffer (Arc<RwLock>)
  ├─→ DisplayPipeline.display_buffer
  ├─→ ViewModel (Qt)
  └─→ QML UI (自动刷新)
```

### 关键数据特性

1. **原始数据 (SensorData)**
   - 最小数据单元
   - 仅包含AD值
   - 经过传感器直接采集

2. **处理后数据 (ProcessedData)**
   - 核心业务数据
   - 包含计算结果
   - 带时间戳和序列号

3. **滤波缓冲区 (FilterBuffer)**
   - 平滑噪声
   - 提高数据质量
   - 支持多种滤波算法

4. **共享缓冲区 (SharedBuffer)**
   - 线程安全
   - 历史数据缓存
   - 内存限制保护

5. **数据库存储**
   - 持久化保存
   - 支持查询分析
   - 自动清理旧数据

这个数据架构设计确保了从传感器到UI显示的高效、可靠、线程安全的数据流转。
