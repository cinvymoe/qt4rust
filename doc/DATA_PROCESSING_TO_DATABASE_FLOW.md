# 数据处理到数据库流程总结

## 概述

本项目采用**三后台管道架构**，实现了数据采集、处理、存储、显示的完全解耦。每个管道独立运行在不同线程，支持独立频率控制和错误处理。

## 整体架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                         数据源层 (Repository)                        │
│  CraneDataRepository → SensorDataSource (模拟/串口/CAN)              │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    管道 1: 采集管道 (CollectionPipeline)              │
│  - 频率: 100ms (10Hz)                                               │
│  - 职责: 从传感器采集原始数据 (SensorData)                            │
│  - 位置: src/pipeline/collection_pipeline.rs                         │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
            ┌───────────────┐               ┌───────────────┐
            │  事件通道      │               │  共享缓冲区    │
            │ (EventChannel) │               │(SharedBuffer) │
            └───────────────┘               └───────────────┘
                    │                               │
                    ▼                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    管道 2: 处理管道 (ProcessPipeline)                 │
│  - 频率: 100ms (10Hz)                                               │
│  - 职责: 验证、计算（力矩百分比、危险状态）                           │
│  - 位置: src/pipeline/process_pipeline.rs                            │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
            ┌───────────────┐               ┌───────────────┐
            │ StorageEvent  │               │  共享缓冲区    │
            │   Sender     │               │(SharedBuffer) │
            └───────────────┘               └───────────────┘
                    │                               │
                    ▼                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    管道 3: 存储管道 (StoragePipeline)                 │
│  - 频率: 1000ms (1Hz，可配置)                                       │
│  - 职责: 批量持久化到 SQLite                                         │
│  - 位置: src/pipeline/storage_pipeline.rs                            │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         数据库层 (SQLite)                            │
│  - 运行时数据表: runtime_data                                        │
│  - 报警记录表: alarm_records                                         │
│  - 传感器数据表: sensor_data                                          │
│  - 位置: src/repositories/sqlite_storage_repository.rs               │
└─────────────────────────────────────────────────────────────────────┘
                                    ▲
                                    │
┌─────────────────────────────────────────────────────────────────────┐
│                    管道 4: 显示管道 (DisplayPipeline)                │
│  - 频率: 500ms (2Hz，主线程)                                         │
│  - 职责: 从共享缓冲区读取数据，通过 Intent 更新 ViewModel              │
│  - 位置: src/pipeline/display_pipeline.rs                           │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Qt/QML UI 层                                 │
│  - ViewModel (MonitoringViewModel)                                   │
│  - QML 界面自动刷新                                                  │
└─────────────────────────────────────────────────────────────────────┘
```

## 核心数据模型

### 1. SensorData (原始传感器数据)

```rust
// src/models/sensor_data.rs
pub struct SensorData {
    pub ad1_load: f64,      // AD1 - 当前载荷（吨）
    pub ad2_radius: f64,    // AD2 - 工作半径（米）
    pub ad3_angle: f64,    // AD3 - 吊臂角度（度）
}
```

### 2. ProcessedData (处理后数据)

```rust
// src/models/processed_data.rs
pub struct ProcessedData {
    pub current_load: f64,        // 当前载荷（吨）
    pub working_radius: f64,      // 工作半径（米）
    pub boom_angle: f64,          // 吊臂角度（度）
    pub moment_percentage: f64,   // 力矩百分比
    pub is_danger: bool,          // 是否危险
    pub validation_error: Option<String>, // 验证错误
    pub timestamp: SystemTime,     // 时间戳
    pub sequence_number: u64,     // 序列号（用于去重）
}
```

### 3. AlarmRecord (报警记录)

```rust
// src/models/alarm_record.rs
pub struct AlarmRecord {
    pub id: Option<i64>,
    pub sequence_number: u64,
    pub timestamp: SystemTime,
    pub alarm_type: AlarmType,    // Warning 或 Danger
    pub current_load: f64,
    pub rated_load: f64,
    pub working_radius: f64,
    pub boom_angle: f64,
    pub moment_percentage: f64,
    pub description: String,
    pub acknowledged: bool,
    pub acknowledged_at: Option<SystemTime>,
}
```

## 数据处理流程详解

### 第一步: 数据采集 (CollectionPipeline)

**文件**: `src/pipeline/collection_pipeline.rs`

```
传感器数据源 → CollectionPipeline (100ms) → SensorData
```

**核心逻辑**:
1. 定时从 `CraneDataRepository` 获取原始传感器数据
2. 支持重试机制 (默认 3 次)
3. 断连检测 (连续失败 10 次触发断连警告)
4. 可选写入 `FilterBuffer` (多速率模式) 或直接处理

**关键代码**:
```rust
// 采集数据带重试
fn collect_with_retry(
    repository: &CraneDataRepository,
    config: &CollectionPipelineConfig,
) -> Result<SensorData, String> {
    for attempt in 0..=config.max_retries {
        match repository.get_latest_sensor_data() {
            Ok(data) => return Ok(data),
            Err(e) => { /* retry */ }
        }
    }
    Err(format!("Failed after {} retries", config.max_retries))
}
```

### 第二步: 数据处理 (ProcessPipeline)

**文件**: `src/pipeline/process_pipeline.rs`

```
SensorData → ProcessPipeline (100ms) → ProcessedData
```

**核心逻辑**:
1. 从 `FilterBuffer` 读取滤波后的传感器数据
2. 使用 `CraneConfig` 进行标定转换 (AD值 → 物理值)
3. 计算力矩百分比
4. 判断危险状态 (力矩 ≥ 90% 为预警，≥ 100% 为报警)
5. 发送 `StorageEvent` 到存储管道
6. 写入显示缓冲区

**关键代码**:
```rust
// 力矩百分比计算
let moment_percentage = (current_load * working_radius) / (rated_load * working_radius) * 100.0;

// 危险判断
let is_danger = moment_percentage >= 100.0;
```

### 第三步: 数据存储 (StoragePipeline)

**文件**: `src/pipeline/storage_pipeline.rs`

```
StorageEvent → StoragePipeline (1000ms) → SQLite Database
```

**核心逻辑**:
1. 通过 `StorageEventReceiver` 接收数据事件
2. 批量积累数据 (默认 batch_size=10)
3. 定时 flush 到数据库 (默认 1000ms)
4. 使用序列号避免重复存储
5. 报警数据立即异步存储

**存储模式**:
- **运行时数据**: 批量存储，1 秒间隔
- **报警数据**: 立即存储，不经过批量队列

### 第四步: 数据显示 (DisplayPipeline)

**文件**: `src/pipeline/display_pipeline.rs`

```
SharedBuffer → DisplayPipeline (500ms) → ViewModel → QML UI
```

**核心逻辑**:
1. 主线程定时调用 `tick()` 方法
2. 从 `SharedBuffer` 读取最新数据
3. 通过 `Intent` 更新 ViewModel
4. QML 界面自动刷新

## 数据库设计

### 表 1: runtime_data (运行时数据)

```sql
CREATE TABLE runtime_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL UNIQUE,  -- 防止重复存储
    timestamp INTEGER NOT NULL,              -- Unix 时间戳
    current_load REAL NOT NULL,              -- 当前载荷（吨）
    working_radius REAL NOT NULL,             -- 工作半径（米）
    boom_angle REAL NOT NULL,                -- 吊臂角度（度）
    moment_percentage REAL NOT NULL,          -- 力矩百分比
    is_danger BOOLEAN NOT NULL,              -- 是否危险
    validation_error TEXT,                   -- 验证错误信息
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 索引
CREATE INDEX idx_runtime_timestamp ON runtime_data(timestamp);
CREATE INDEX idx_runtime_sequence ON runtime_data(sequence_number);
```

### 表 2: alarm_records (报警记录)

```sql
CREATE TABLE alarm_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    alarm_type TEXT NOT NULL,                -- 'warning' 或 'danger'
    current_load REAL NOT NULL,
    rated_load REAL NOT NULL,
    working_radius REAL NOT NULL,
    boom_angle REAL NOT NULL,
    boom_length REAL NOT NULL,
    moment_percentage REAL NOT NULL,
    description TEXT,
    acknowledged BOOLEAN NOT NULL DEFAULT 0,  -- 是否已确认
    acknowledged_at INTEGER,                  -- 确认时间
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 索引
CREATE INDEX idx_alarm_timestamp ON alarm_records(timestamp);
CREATE INDEX idx_alarm_acknowledged ON alarm_records(acknowledged);
```

### 表 3: sensor_data (传感器原始数据)

```sql
CREATE TABLE sensor_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    sequence_number INTEGER NOT NULL UNIQUE,
    ad1_load REAL NOT NULL,
    ad2_radius REAL NOT NULL,
    ad3_angle REAL NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- 索引
CREATE INDEX idx_sensor_timestamp ON sensor_data(timestamp);
CREATE INDEX idx_sensor_sequence ON sensor_data(sequence_number);
```

## Repository 模式

### StorageRepository Trait (抽象接口)

**文件**: `src/repositories/storage_repository.rs`

```rust
#[async_trait]
pub trait StorageRepository: Send + Sync {
    // 批量存储运行数据
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String>;
    
    // 存储单条报警记录
    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String>;
    
    // 查询最近的运行数据
    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String>;
    
    // 查询未确认的报警
    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String>;
    
    // 确认报警
    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String>;
    
    // 获取最后存储的序列号
    async fn get_last_stored_sequence(&self) -> Result<u64, String>;
    
    // 健康检查
    async fn health_check(&self) -> Result<(), String>;
    
    // 清理旧数据
    async fn purge_old_records(&self, max_records: usize, purge_threshold: usize) -> Result<usize, String>;
    
    // 清理旧报警记录
    async fn purge_old_alarms(&self, alarm_max_records: usize, alarm_purge_threshold: usize) -> Result<usize, String>;
}
```

### SqliteStorageRepository (SQLite 实现)

**文件**: `src/repositories/sqlite_storage_repository.rs`

**核心特性**:
- 使用 `rusqlite` 库
- 使用 `tokio` 异步运行时
- 事务批处理提升性能
- `Arc<Mutex<Connection>>` 保证线程安全

**关键方法**:
```rust
// 批量存储 (使用事务)
async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String> {
    conn.execute("BEGIN TRANSACTION", [])?;
    for item in data {
        conn.execute("INSERT OR IGNORE INTO runtime_data ...", params![...])?;
    }
    conn.execute("COMMIT", [])?;
}

// 报警存储 (立即写入)
async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String> {
    // 立即写入，不经过批量队列
    conn.execute("INSERT INTO alarm_records ...", params![...])?;
    Ok(conn.last_insert_rowid())
}
```

### MockStorageRepository (测试实现)

**文件**: `src/repositories/mock_storage_repository.rs`

用于单元测试，不涉及真实数据库操作。

## 事件驱动架构

### StorageEvent (存储事件)

**文件**: `src/pipeline/mod.rs`

```rust
#[derive(Debug, Clone)]
pub enum StorageEvent {
    NewData(Vec<ProcessedData>),   // 新数据可用于存储
    Alarm(ProcessedData),           // 报警触发
    AlarmCleared,                   // 报警解除
    Shutdown,                       // 请求关闭
}
```

### Event Channel (事件通道)

**文件**: `src/pipeline/event_channel.rs`

```rust
pub struct StorageEventSender {
    data_tx: ...,
    shutdown_tx: ...,
}

pub struct StorageEventReceiver {
    data_rx: ...,
    shutdown_rx: ...,
}

// 创建通道
pub fn create_storage_channels(max_size: usize) -> (StorageEventSender, StorageEventReceiver);
```

### SensorDataEventChannel (传感器数据通道)

**文件**: `src/pipeline/sensor_data_event_channel.rs`

用于原始传感器数据的存储，支持双表存储 (sensor_data + runtime_data)。

## 关键组件

### 1. StorageQueue (存储队列)

**文件**: `src/pipeline/storage_queue.rs`

```rust
pub struct StorageQueue {
    queue: Arc<Mutex<VecDeque<ProcessedData>>>,
    last_stored_sequence: Arc<Mutex<u64>>,
}
```

**功能**:
- 批量缓存待存储数据
- 追踪已存储的序列号
- 自动过滤重复数据

### 2. SharedBuffer (共享缓冲区)

**文件**: `src/pipeline/shared_buffer.rs`

```rust
pub type SharedBuffer = Arc<RwLock<ProcessedDataBuffer>>;

pub struct ProcessedDataBuffer {
    latest: Option<ProcessedData>,
    history: VecDeque<ProcessedData>,
    max_history_size: usize,
    stats: BufferStats,
}
```

**功能**:
- 线程安全的数据共享
- 存储最新数据和历史队列
- 统计信息 (采集次数、错误次数)

### 3. FilterBuffer (滤波缓冲区)

**文件**: `src/pipeline/filter_buffer.rs`

用于多速率架构，对传感器数据进行滤波处理。

## 数据流时序图

```
时间轴
  │
  │  采集管道 ─── 处理管道 ─── 存储管道 ─── 显示管道
  │    │            │            │            │
  │    ▼            │            │            │
  │ SensorData      │            │            │
  │    │            │            │            │
  │    ▼            │            │            │
  │ 处理完成 ───────►│            │            │
  │                 ▼            │            │
  │          ProcessedData      │            │
  │                 │            │            │
  │                 ▼            │            │
  │          ┌────────────┐     │            │
  │          │StorageEvent│     │            │
  │          └────────────┘     │            │
  │                 │            │            │
  │                 └───────────►│            │
  │                              ▼            │
  │                       积累 batch         │
  │                              │            │
  │                              ▼ (1000ms)   │
  │                       SQLite Batch Insert │
  │                              │            │
  │                              ▼            │
  │                       SharedBuffer       │
  │                              │            │
  │                              ▼ (500ms)    │
  │                       ViewModel ────────►│
  │                                          ▼
  │                                    QML UI 更新
```

## 配置管理

### PipelineConfig

**文件**: `src/config/pipeline_config.rs`

```rust
// 采集管道配置
pub struct CollectionConfig {
    pub interval_ms: u64,          // 采集间隔 (默认 100ms)
    pub max_retries: u32,          // 最大重试次数
    pub retry_delay_ms: u64,       // 重试延迟
    pub disconnect_threshold: u32, // 断连阈值
}

// 存储管道配置
pub struct StorageConfig {
    pub interval_ms: u64,           // 存储间隔 (默认 5000ms)
    pub batch_size: usize,         // 批量大小
    pub max_retries: u32,          // 最大重试次数
    pub max_queue_size: usize,      // 队列最大容量
    pub max_records: usize,         // 最大记录数 (0=不限制)
    pub purge_threshold: usize,     // 清理阈值
    pub alarm_max_records: usize,   // 报警最大记录数
}

// 显示管道配置
pub struct DisplayConfig {
    pub interval_ms: u64,          // 显示间隔 (默认 500ms)
    pub pipeline_size: usize,      // 管道大小
    pub batch_size: usize,         // 批量大小
}
```

## 错误处理机制

### 1. 采集失败重试

```rust
// collection_pipeline.rs
for attempt in 0..=config.max_retries {
    match repository.get_latest_sensor_data() {
        Ok(data) => return Ok(data),
        Err(e) => {
            if attempt < config.max_retries {
                thread::sleep(config.retry_delay);
            }
        }
    }
}
```

### 2. 存储失败重试

```rust
// storage_pipeline.rs
let result = with_retry(
    &RetryConfig {
        max_retries: config.max_retries,
        base_delay: config.retry_delay,
        ..Default::default()
    },
    || async { repo.save_runtime_data_batch(&data).await },
).await;
```

### 3. 断连检测

```rust
if consecutive_failures >= config.disconnect_threshold {
    tracing::error!("Sensor disconnected (threshold reached)");
}
```

## 文件结构总结

```
src/
├── pipeline/
│   ├── mod.rs                    # 模块定义和 StorageEvent
│   ├── collection_pipeline.rs    # 采集管道
│   ├── process_pipeline.rs       # 处理管道
│   ├── storage_pipeline.rs       # 存储管道
│   ├── display_pipeline.rs       # 显示管道
│   ├── event_channel.rs          # 存储事件通道
│   ├── sensor_data_event_channel.rs # 传感器数据事件通道
│   ├── shared_buffer.rs          # 共享缓冲区
│   ├── filter_buffer.rs          # 滤波缓冲区
│   ├── storage_queue.rs          # 存储队列
│   └── retry_policy.rs          # 重试策略
│
├── repositories/
│   ├── mod.rs                   # 模块导出
│   ├── storage_repository.rs     # StorageRepository trait
│   ├── sqlite_storage_repository.rs # SQLite 实现
│   ├── mock_storage_repository.rs   # 测试 Mock
│   ├── sensor_data_repository.rs    # 传感器数据仓库
│   └── crane_data_repository.rs     # 起重机数据仓库
│
├── models/
│   ├── mod.rs                   # 模块导出
│   ├── sensor_data.rs           # SensorData 模型
│   ├── processed_data.rs        # ProcessedData 模型
│   ├── alarm_record.rs          # AlarmRecord 模型
│   ├── sensor_calibration.rs    # 标定参数
│   ├── rated_load_table.rs      # 额定载荷表
│   └── crane_config.rs          # 起重机配置
│
└── config/
    ├── mod.rs
    ├── config_manager.rs        # 配置管理器
    ├── pipeline_config.rs       # 管道配置
    ├── calibration_manager.rs   # 标定管理器
    └── load_table_manager.rs    # 载荷表管理器
```

## 总结

数据处理到数据库的完整流程：

1. **采集**: `CraneDataRepository` → `SensorData` (100ms)
2. **滤波**: `FilterBuffer` 对数据进行滤波处理
3. **处理**: `ProcessPipeline` → 计算力矩、判断危险 → `ProcessedData` (100ms)
4. **事件**: 通过 `StorageEventSender` 发送存储事件
5. **存储**: `StoragePipeline` 批量积累 → SQLite (1000ms)
6. **显示**: `DisplayPipeline` 从 `SharedBuffer` 读取 → UI 更新 (500ms)

整个架构采用**解耦设计**，各管道独立运行，通过**事件通道**和**共享缓冲区**通信，支持**灵活的配置**和**错误处理**。