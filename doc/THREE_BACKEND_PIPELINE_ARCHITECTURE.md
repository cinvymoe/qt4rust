# 三后台管道架构设计文档

## 1. 概述

本文档定义了 Crane 监控系统的三后台管道架构，将数据采集、处理、存储和显示完全解耦，实现独立的频率控制和错误处理。

### 1.1 设计目标

- **解耦**: 采集、存储、显示三个管道完全独立
- **独立频率**: 每个管道可独立配置运行频率
- **容错性**: 单个管道故障不影响其他管道
- **线程安全**: 使用 Rust 的并发原语保证数据安全
- **MVI 集成**: 与现有 MVI 架构无缝集成

### 1.2 核心原则

1. **单向数据流**: 采集 → 处理 → 存储 → 显示
2. **管道隔离**: 每个管道独立线程，独立错误处理
3. **数据不可变**: 使用 Arc 共享只读数据
4. **Qt 线程安全**: 显示管道在主线程通过 Intent 更新 ViewModel

## 2. 架构概览

### 2.1 三管道架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                         主线程 (Qt)                              │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              ViewModel (MonitoringViewModel)             │  │
│  │         通过 Intent/Reducer 更新状态                      │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            ↑                                     │
│                            │ Intent (主线程)                     │
│                            │                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           Pipeline 3: 显示管道 (主线程)                  │  │
│  │  - 频率: 100ms (10Hz，可配置)                            │  │
│  │  - 职责: 从共享缓冲区读取，通过 Intent 更新 ViewModel    │  │
│  │  - 错误处理: 降级显示，使用缓存数据                      │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↑
                            │ 读取 (Arc<RwLock>)
                            │
┌─────────────────────────────────────────────────────────────────┐
│                    共享缓冲区 (线程安全)                         │
│  Arc<RwLock<ProcessedDataBuffer>>                               │
│  - 最新处理后数据                                                │
│  - 历史数据队列 (VecDeque, 容量限制)                            │
│  - 统计信息 (采集次数、错误次数等)                              │
└─────────────────────────────────────────────────────────────────┘
                            ↑
                            │ 写入 (Arc<RwLock>)
                            │
┌─────────────────────────────────────────────────────────────────┐
│                      后台线程 2                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           Pipeline 2: 存储管道                            │  │
│  │  - 频率: 1000ms (1Hz，可配置)                            │  │
│  │  - 职责: 从共享缓冲区读取，持久化到 SQLite               │  │
│  │  - 错误处理: 重试机制，失败记录到日志                    │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↑
                            │ 读取 (mpsc channel)
                            │
┌─────────────────────────────────────────────────────────────────┐
│                      后台线程 1                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │      Pipeline 1: 采集与处理管道                          │  │
│  │  - 频率: 100ms (10Hz，可配置)                            │  │
│  │  - 职责:                                                  │  │
│  │    1. 从传感器采集原始数据 (SensorData)                  │  │
│  │    2. 数据验证和处理                                      │  │
│  │    3. 计算派生数据 (力矩百分比、危险状态等)              │  │
│  │    4. 写入共享缓冲区                                      │  │
│  │  - 错误处理: 重试、降级、断连检测                        │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            ↑
                            │
┌─────────────────────────────────────────────────────────────────┐
│                   数据源层 (Repository)                          │
│  CraneDataRepository → SensorDataSource                         │
│  (模拟数据 / 串口 / CAN总线)                                    │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 数据流向

```
原始传感器数据 (SensorData)
    ↓ 100ms 采集
Pipeline 1: 采集与处理
    ↓ 验证、计算
处理后数据 (ProcessedData)
    ↓ 写入共享缓冲区
    ├─→ Pipeline 2: 存储 (1000ms)
    │       ↓ 持久化
    │   SQLite 数据库
    │
    └─→ Pipeline 3: 显示 (100ms)
            ↓ 读取最新数据
        通过 Intent 更新 ViewModel
            ↓
        QML UI 自动刷新
```

## 3. 数据模型定义

### 3.1 原始传感器数据 (现有)

```rust
// src/models/sensor_data.rs (已存在)

/// 原始传感器数据
#[derive(Debug, Clone, PartialEq)]
pub struct SensorData {
    /// AD1 - 当前载荷（吨）
    pub ad1_load: f64,
    /// AD2 - 工作半径（米）
    pub ad2_radius: f64,
    /// AD3 - 吊臂角度（度）
    pub ad3_angle: f64,
    /// 额定载荷（吨）
    pub rated_load: f64,
    /// 臂长（米）
    pub boom_length: f64,
}
```

### 3.2 处理后数据 (新增)

```rust
// src/models/processed_data.rs (新增)

use std::time::SystemTime;
use super::sensor_data::SensorData;

/// 处理后的数据（包含计算结果）
#[derive(Debug, Clone)]
pub struct ProcessedData {
    /// 原始传感器数据
    pub raw_data: SensorData,
    
    /// 力矩百分比（计算得出）
    pub moment_percentage: f64,
    
    /// 是否处于危险状态
    pub is_danger: bool,
    
    /// 数据验证结果
    pub validation_error: Option<String>,
    
    /// 采集时间戳
    pub timestamp: SystemTime,
    
    /// 数据序列号（用于追踪）
    pub sequence_number: u64,
}

impl ProcessedData {
    /// 从原始传感器数据创建处理后数据
    pub fn from_sensor_data(raw_data: SensorData, sequence_number: u64) -> Self {
        // 计算力矩百分比
        let moment_percentage = Self::calculate_moment_percentage(&raw_data);
        
        // 判断危险状态
        let is_danger = moment_percentage >= 90.0;
        
        // 验证数据
        let validation_error = raw_data.validate().err();
        
        Self {
            raw_data,
            moment_percentage,
            is_danger,
            validation_error,
            timestamp: SystemTime::now(),
            sequence_number,
        }
    }
    
    fn calculate_moment_percentage(data: &SensorData) -> f64 {
        let current_moment = data.ad1_load * data.ad2_radius;
        let rated_moment = data.rated_load * data.ad2_radius;
        
        if rated_moment > 0.0 {
            (current_moment / rated_moment) * 100.0
        } else {
            0.0
        }
    }
}
```

### 3.3 共享缓冲区

```rust
// src/pipeline/shared_buffer.rs (新增)

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use crate::models::processed_data::ProcessedData;

/// 共享数据缓冲区
#[derive(Debug)]
pub struct ProcessedDataBuffer {
    /// 最新的处理后数据
    latest: Option<ProcessedData>,
    
    /// 历史数据队列（用于图表显示）
    history: VecDeque<ProcessedData>,
    
    /// 历史数据最大容量
    max_history_size: usize,
    
    /// 统计信息
    stats: BufferStats,
}

#[derive(Debug, Default)]
pub struct BufferStats {
    /// 总采集次数
    pub total_collections: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub error_count: u64,
    /// 最后更新时间
    pub last_update_time: Option<std::time::SystemTime>,
}

impl ProcessedDataBuffer {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            latest: None,
            history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            stats: BufferStats::default(),
        }
    }
    
    /// 写入新数据
    pub fn push(&mut self, data: ProcessedData) {
        // 更新最新数据
        self.latest = Some(data.clone());
        
        // 添加到历史队列
        if self.history.len() >= self.max_history_size {
            self.history.pop_front();
        }
        self.history.push_back(data);
        
        // 更新统计
        self.stats.total_collections += 1;
        self.stats.success_count += 1;
        self.stats.last_update_time = Some(std::time::SystemTime::now());
    }
    
    /// 读取最新数据
    pub fn get_latest(&self) -> Option<ProcessedData> {
        self.latest.clone()
    }
    
    /// 读取历史数据
    pub fn get_history(&self, count: usize) -> Vec<ProcessedData> {
        self.history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// 记录错误
    pub fn record_error(&mut self) {
        self.stats.error_count += 1;
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> &BufferStats {
        &self.stats
    }
}

/// 线程安全的共享缓冲区类型
pub type SharedBuffer = Arc<RwLock<ProcessedDataBuffer>>;
```

## 4. Pipeline 1: 采集与处理管道

### 4.1 管道配置

```rust
// src/pipeline/collection_pipeline.rs (新增)

use std::time::Duration;

/// 采集管道配置
#[derive(Debug, Clone)]
pub struct CollectionPipelineConfig {
    /// 采集间隔
    pub interval: Duration,
    
    /// 失败重试次数
    pub max_retries: u32,
    
    /// 重试延迟
    pub retry_delay: Duration,
    
    /// 断连检测阈值（连续失败次数）
    pub disconnect_threshold: u32,
}

impl Default for CollectionPipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),  // 100ms = 10Hz
            max_retries: 3,
            retry_delay: Duration::from_millis(10),
            disconnect_threshold: 10,
        }
    }
}
```

### 4.2 管道实现

```rust
// src/pipeline/collection_pipeline.rs (续)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use crate::repositories::CraneDataRepository;
use crate::models::processed_data::ProcessedData;
use super::shared_buffer::SharedBuffer;

/// 采集与处理管道
pub struct CollectionPipeline {
    config: CollectionPipelineConfig,
    repository: Arc<CraneDataRepository>,
    buffer: SharedBuffer,
    running: Arc<AtomicBool>,
    sequence_number: Arc<AtomicU64>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CollectionPipeline {
    pub fn new(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        buffer: SharedBuffer,
    ) -> Self {
        Self {
            config,
            repository,
            buffer,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
        }
    }
    
    /// 启动管道
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            eprintln!("[WARN] Collection pipeline already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let config = self.config.clone();
        let repository = Arc::clone(&self.repository);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let sequence_number = Arc::clone(&self.sequence_number);
        
        let handle = thread::spawn(move || {
            eprintln!("[INFO] Collection pipeline started");
            let mut consecutive_failures = 0;
            
            while running.load(Ordering::Relaxed) {
                let start_time = std::time::Instant::now();
                
                // 采集数据
                match Self::collect_with_retry(&repository, &config) {
                    Ok(sensor_data) => {
                        // 重置失败计数
                        consecutive_failures = 0;
                        
                        // 处理数据
                        let seq = sequence_number.fetch_add(1, Ordering::Relaxed);
                        let processed = ProcessedData::from_sensor_data(sensor_data, seq);
                        
                        // 写入共享缓冲区
                        if let Ok(mut buf) = buffer.write() {
                            buf.push(processed);
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        eprintln!("[ERROR] Collection failed: {} (consecutive: {})", 
                                  e, consecutive_failures);
                        
                        // 记录错误
                        if let Ok(mut buf) = buffer.write() {
                            buf.record_error();
                        }
                        
                        // 检测断连
                        if consecutive_failures >= config.disconnect_threshold {
                            eprintln!("[ERROR] Sensor disconnected (threshold reached)");
                            // TODO: 触发断连事件
                        }
                    }
                }
                
                // 控制采集频率
                let elapsed = start_time.elapsed();
                if elapsed < config.interval {
                    thread::sleep(config.interval - elapsed);
                }
            }
            
            eprintln!("[INFO] Collection pipeline stopped");
        });
        
        self.handle = Some(handle);
    }
    
    /// 停止管道
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
    
    /// 带重试的数据采集
    fn collect_with_retry(
        repository: &CraneDataRepository,
        config: &CollectionPipelineConfig,
    ) -> Result<crate::models::SensorData, String> {
        let mut last_error = String::new();
        
        for attempt in 0..=config.max_retries {
            match repository.get_latest_sensor_data() {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = e;
                    if attempt < config.max_retries {
                        thread::sleep(config.retry_delay);
                    }
                }
            }
        }
        
        Err(format!("Failed after {} retries: {}", config.max_retries, last_error))
    }
}

impl Drop for CollectionPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}
```

## 5. Pipeline 2: 存储管道（改进版）

### 5.1 数据库表设计

```sql
-- 运行数据表（定时批量存储）
CREATE TABLE IF NOT EXISTS runtime_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL UNIQUE,  -- 数据序列号，防止重复
    timestamp INTEGER NOT NULL,               -- 时间戳（Unix 时间）
    current_load REAL NOT NULL,               -- 当前载荷（吨）
    rated_load REAL NOT NULL,                 -- 额定载荷（吨）
    working_radius REAL NOT NULL,             -- 工作半径（米）
    boom_angle REAL NOT NULL,                 -- 吊臂角度（度）
    boom_length REAL NOT NULL,                -- 臂长（米）
    moment_percentage REAL NOT NULL,          -- 力矩百分比
    is_danger BOOLEAN NOT NULL,               -- 是否危险
    validation_error TEXT,                    -- 验证错误信息
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_runtime_timestamp ON runtime_data(timestamp);
CREATE INDEX IF NOT EXISTS idx_runtime_sequence ON runtime_data(sequence_number);

-- 报警信息表（异步回调立即存储）
CREATE TABLE IF NOT EXISTS alarm_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL,         -- 关联的数据序列号
    timestamp INTEGER NOT NULL,               -- 报警时间戳
    alarm_type TEXT NOT NULL,                 -- 报警类型：warning/danger
    current_load REAL NOT NULL,
    rated_load REAL NOT NULL,
    working_radius REAL NOT NULL,
    boom_angle REAL NOT NULL,
    boom_length REAL NOT NULL,
    moment_percentage REAL NOT NULL,
    description TEXT,                         -- 报警描述
    acknowledged BOOLEAN NOT NULL DEFAULT 0,  -- 是否已确认
    acknowledged_at INTEGER,                  -- 确认时间
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_alarm_timestamp ON alarm_records(timestamp);
CREATE INDEX IF NOT EXISTS idx_alarm_type ON alarm_records(alarm_type);
CREATE INDEX IF NOT EXISTS idx_alarm_acknowledged ON alarm_records(acknowledged);
```

### 5.2 管道配置

```rust
// src/pipeline/storage_pipeline.rs (新增)

use std::time::Duration;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// 存储管道配置
#[derive(Debug, Clone)]
pub struct StoragePipelineConfig {
    /// 存储间隔（运行数据）
    pub interval: Duration,
    
    /// 批量存储大小
    pub batch_size: usize,
    
    /// 失败重试次数
    pub max_retries: u32,
    
    /// 重试延迟
    pub retry_delay: Duration,
    
    /// 管道队列最大容量
    pub max_queue_size: usize,
}

impl Default for StoragePipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),  // 1秒存储一次
            batch_size: 10,
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            max_queue_size: 1000,  // 最多缓存 1000 条数据
        }
    }
}
```

### 5.3 存储管道队列

```rust
// src/pipeline/storage_queue.rs (新增)

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::models::processed_data::ProcessedData;

/// 存储队列（线程安全）
pub struct StorageQueue {
    queue: Arc<Mutex<VecDeque<ProcessedData>>>,
    max_size: usize,
    last_stored_sequence: Arc<Mutex<u64>>,
}

impl StorageQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
            last_stored_sequence: Arc::new(Mutex::new(0)),
        }
    }
    
    /// 添加数据到队列
    pub fn push(&self, data: ProcessedData) -> Result<(), String> {
        let mut queue = self.queue.lock()
            .map_err(|e| format!("Failed to lock queue: {}", e))?;
        
        // 检查是否已存储（避免重复）
        if let Ok(last_seq) = self.last_stored_sequence.lock() {
            if data.sequence_number <= *last_seq {
                return Ok(()); // 已存储，跳过
            }
        }
        
        // 检查队列容量
        if queue.len() >= self.max_size {
            eprintln!("[WARN] Storage queue full, dropping oldest data");
            queue.pop_front();
        }
        
        queue.push_back(data);
        Ok(())
    }
    
    /// 批量取出数据（不删除）
    pub fn peek_batch(&self, count: usize) -> Vec<ProcessedData> {
        if let Ok(queue) = self.queue.lock() {
            queue.iter().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// 删除已存储的数据
    pub fn remove_stored(&self, count: usize, max_sequence: u64) -> Result<(), String> {
        let mut queue = self.queue.lock()
            .map_err(|e| format!("Failed to lock queue: {}", e))?;
        
        // 删除前 count 条数据
        for _ in 0..count.min(queue.len()) {
            queue.pop_front();
        }
        
        // 更新最后存储的序列号
        if let Ok(mut last_seq) = self.last_stored_sequence.lock() {
            *last_seq = max_sequence;
        }
        
        Ok(())
    }
    
    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.queue.lock().map(|q| q.len()).unwrap_or(0)
    }
    
    /// 获取最后存储的序列号
    pub fn last_stored_sequence(&self) -> u64 {
        self.last_stored_sequence.lock()
            .map(|seq| *seq)
            .unwrap_or(0)
    }
}
```

### 5.4 数据库操作层（解耦设计）

**核心原则**：数据库操作通过 trait 抽象，实现依赖倒置。

```rust
// src/repositories/storage_repository.rs (新增)

use async_trait::async_trait;
use crate::models::processed_data::ProcessedData;
use crate::models::alarm_record::AlarmRecord;

/// 存储仓库 trait（抽象接口）
#[async_trait]
pub trait StorageRepository: Send + Sync {
    /// 批量存储运行数据
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String>;
    
    /// 存储单条报警记录
    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String>;
    
    /// 查询最近的运行数据
    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String>;
    
    /// 查询未确认的报警
    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String>;
    
    /// 确认报警
    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String>;
    
    /// 获取最后存储的序列号
    async fn get_last_stored_sequence(&self) -> Result<u64, String>;
    
    /// 健康检查
    async fn health_check(&self) -> Result<(), String>;
}
```

**优势**：
- ✅ 易于测试（可以 mock Repository）
- ✅ 支持多种数据库（SQLite、PostgreSQL、MySQL）
- ✅ 符合依赖倒置原则
- ✅ 业务逻辑与数据访问分离

详细设计参见：`doc/STORAGE_DECOUPLING_DESIGN.md`

### 5.5 存储管道实现（解耦版）

### 5.5 存储管道实现（解耦版）

```rust
// src/pipeline/storage_pipeline.rs（解耦版本）

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tokio::runtime::Runtime;
use super::storage_queue::StorageQueue;
use crate::repositories::storage_repository::StorageRepository;  // 依赖抽象
use super::shared_buffer::SharedBuffer;

/// 存储管道（解耦版本）
pub struct StoragePipeline {
    config: StoragePipelineConfig,
    storage_queue: Arc<StorageQueue>,
    repository: Arc<dyn StorageRepository>,  // 依赖抽象接口，而非具体实现
    buffer: SharedBuffer,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    tokio_runtime: Arc<Runtime>,
}

impl StoragePipeline {
    /// 创建存储管道（依赖注入）
    pub fn new(
        config: StoragePipelineConfig,
        repository: Arc<dyn StorageRepository>,  // 注入抽象接口
        buffer: SharedBuffer,
    ) -> Result<Self, String> {
        let storage_queue = Arc::new(StorageQueue::new(config.max_queue_size));
        let tokio_runtime = Arc::new(
            Runtime::new().map_err(|e| format!("Failed to create Tokio runtime: {}", e))?
        );
        
        Ok(Self {
            config,
            storage_queue,
            repository,
            buffer,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            tokio_runtime,
        })
    }
    
    /// 启动管道
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            eprintln!("[WARN] Storage pipeline already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let config = self.config.clone();
        let storage_queue = Arc::clone(&self.storage_queue);
        let repository = Arc::clone(&self.repository);  // 克隆 trait object
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let tokio_runtime = Arc::clone(&self.tokio_runtime);
        
        let handle = thread::spawn(move || {
            eprintln!("[INFO] Storage pipeline started");
            
            while running.load(Ordering::Relaxed) {
                let start_time = std::time::Instant::now();
                
                // 1. 从共享缓冲区读取新数据
                if let Ok(buf) = buffer.read() {
                    let last_seq = storage_queue.last_stored_sequence();
                    let new_data = buf.get_history(config.batch_size)
                        .into_iter()
                        .filter(|d| d.sequence_number > last_seq)
                        .collect::<Vec<_>>();
                    
                    for data in new_data {
                        if let Err(e) = storage_queue.push(data) {
                            eprintln!("[ERROR] Failed to push to storage queue: {}", e);
                        }
                    }
                }
                
                // 2. 从队列取出数据批量存储
                let data_to_store = storage_queue.peek_batch(config.batch_size);
                
                if !data_to_store.is_empty() {
                    let max_sequence = data_to_store.iter()
                        .map(|d| d.sequence_number)
                        .max()
                        .unwrap_or(0);
                    
                    // 异步存储到数据库（通过抽象接口）
                    let repository_clone = Arc::clone(&repository);
                    let storage_queue_clone = Arc::clone(&storage_queue);
                    let data_clone = data_to_store.clone();
                    let count = data_to_store.len();
                    
                    tokio_runtime.spawn(async move {
                        // 调用抽象接口方法
                        match repository_clone.save_runtime_data_batch(&data_clone).await {
                            Ok(saved_count) => {
                                eprintln!("[INFO] Saved {} records", saved_count);
                                // 存储成功，从队列删除
                                if let Err(e) = storage_queue_clone.remove_stored(count, max_sequence) {
                                    eprintln!("[ERROR] Failed to remove stored data: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Failed to save runtime data: {}", e);
                            }
                        }
                    });
                }
                
                // 3. 控制存储频率
                let elapsed = start_time.elapsed();
                if elapsed < config.interval {
                    thread::sleep(config.interval - elapsed);
                }
            }
            
            eprintln!("[INFO] Storage pipeline stopped");
        });
        
        self.handle = Some(handle);
    }
    
    /// 停止管道
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
    
    /// 异步回调：立即存储报警记录（通过抽象接口）
    pub fn save_alarm_async(&self, data: ProcessedData) {
        let repository = Arc::clone(&self.repository);
        
        self.tokio_runtime.spawn(async move {
            // 调用抽象接口方法
            match repository.save_alarm_record(&data).await {
                Ok(alarm_id) => {
                    eprintln!("[INFO] Alarm saved with id: {}", alarm_id);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to save alarm record: {}", e);
                }
            }
        });
    }
}

impl Drop for StoragePipeline {
    fn drop(&mut self) {
        self.stop();
    }
}
```
use std::sync::{Arc, Mutex};
use crate::models::processed_data::ProcessedData;

/// SQLite 数据源
pub struct SqliteDataSource {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteDataSource {
    /// 创建数据源并初始化表
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let source = Self {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        source.init_tables()?;
        Ok(source)
    }
    
    /// 初始化数据库表
    fn init_tables(&self) -> Result<(), String> {
        let conn = self.connection.lock()
            .map_err(|e| format!("Failed to lock connection: {}", e))?;
        
        // 创建运行数据表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS runtime_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sequence_number INTEGER NOT NULL UNIQUE,
                timestamp INTEGER NOT NULL,
                current_load REAL NOT NULL,
                rated_load REAL NOT NULL,
                working_radius REAL NOT NULL,
                boom_angle REAL NOT NULL,
                boom_length REAL NOT NULL,
                moment_percentage REAL NOT NULL,
                is_danger BOOLEAN NOT NULL,
                validation_error TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        ).map_err(|e| format!("Failed to create runtime_data table: {}", e))?;
        
        // 创建报警信息表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS alarm_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sequence_number INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                alarm_type TEXT NOT NULL,
                current_load REAL NOT NULL,
                rated_load REAL NOT NULL,
                working_radius REAL NOT NULL,
                boom_angle REAL NOT NULL,
                boom_length REAL NOT NULL,
                moment_percentage REAL NOT NULL,
                description TEXT,
                acknowledged BOOLEAN NOT NULL DEFAULT 0,
                acknowledged_at INTEGER,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        ).map_err(|e| format!("Failed to create alarm_records table: {}", e))?;
        
        // 创建索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_runtime_timestamp ON runtime_data(timestamp)", [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_runtime_sequence ON runtime_data(sequence_number)", [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_alarm_timestamp ON alarm_records(timestamp)", [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
        
        eprintln!("[INFO] Database tables initialized");
        Ok(())
    }
    
    /// 批量存储运行数据（使用事务）
    pub fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<(), String> {
        if data.is_empty() {
            return Ok(());
        }
        
        let conn = self.connection.lock()
            .map_err(|e| format!("Failed to lock connection: {}", e))?;
        
        // 开始事务
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;
        
        for item in data {
            let timestamp = item.timestamp.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            
            let result = conn.execute(
                "INSERT OR IGNORE INTO runtime_data 
                 (sequence_number, timestamp, current_load, rated_load, working_radius, 
                  boom_angle, boom_length, moment_percentage, is_danger, validation_error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    item.sequence_number as i64,
                    timestamp,
                    item.raw_data.ad1_load,
                    item.raw_data.rated_load,
                    item.raw_data.ad2_radius,
                    item.raw_data.ad3_angle,
                    item.raw_data.boom_length,
                    item.moment_percentage,
                    item.is_danger,
                    item.validation_error.as_ref().map(|s| s.as_str()),
                ],
            );
            
            if let Err(e) = result {
                // 回滚事务
                let _ = conn.execute("ROLLBACK", []);
                return Err(format!("Failed to insert runtime data: {}", e));
            }
        }
        
        // 提交事务
        conn.execute("COMMIT", [])
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
        
        eprintln!("[INFO] Saved {} runtime records to database", data.len());
        Ok(())
    }
    
    /// 异步存储报警记录（立即存储）
    pub fn save_alarm_record(&self, data: &ProcessedData) -> Result<(), String> {
        let conn = self.connection.lock()
            .map_err(|e| format!("Failed to lock connection: {}", e))?;
        
        let timestamp = data.timestamp.duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        let alarm_type = if data.moment_percentage >= 100.0 {
            "danger"
        } else {
            "warning"
        };
        
        let description = format!(
            "力矩百分比 {:.1}% 超过阈值，当前载荷 {:.1}t，工作半径 {:.1}m",
            data.moment_percentage,
            data.raw_data.ad1_load,
            data.raw_data.ad2_radius
        );
        
        conn.execute(
            "INSERT INTO alarm_records 
             (sequence_number, timestamp, alarm_type, current_load, rated_load, 
              working_radius, boom_angle, boom_length, moment_percentage, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                data.sequence_number as i64,
                timestamp,
                alarm_type,
                data.raw_data.ad1_load,
                data.raw_data.rated_load,
                data.raw_data.ad2_radius,
                data.raw_data.ad3_angle,
                data.raw_data.boom_length,
                data.moment_percentage,
                description,
            ],
        ).map_err(|e| format!("Failed to insert alarm record: {}", e))?;
        
        eprintln!("[INFO] Saved alarm record: {} at {:.1}%", alarm_type, data.moment_percentage);
        Ok(())
    }
}
```

### 5.5 存储管道实现（改进版）

```rust
// src/pipeline/storage_pipeline.rs (续)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tokio::runtime::Runtime;
use super::storage_queue::StorageQueue;
use crate::data_sources::sqlite_data_source::SqliteDataSource;
use super::shared_buffer::SharedBuffer;

/// 存储管道
pub struct StoragePipeline {
    config: StoragePipelineConfig,
    storage_queue: Arc<StorageQueue>,
    db_source: Arc<SqliteDataSource>,
    buffer: SharedBuffer,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    tokio_runtime: Arc<Runtime>,
}

impl StoragePipeline {
    pub fn new(
        config: StoragePipelineConfig,
        db_path: &str,
        buffer: SharedBuffer,
    ) -> Result<Self, String> {
        let storage_queue = Arc::new(StorageQueue::new(config.max_queue_size));
        let db_source = Arc::new(SqliteDataSource::new(db_path)?);
        let tokio_runtime = Arc::new(
            Runtime::new().map_err(|e| format!("Failed to create Tokio runtime: {}", e))?
        );
        
        Ok(Self {
            config,
            storage_queue,
            db_source,
            buffer,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            tokio_runtime,
        })
    }
    
    /// 启动管道
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            eprintln!("[WARN] Storage pipeline already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let config = self.config.clone();
        let storage_queue = Arc::clone(&self.storage_queue);
        let db_source = Arc::clone(&self.db_source);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let tokio_runtime = Arc::clone(&self.tokio_runtime);
        
        let handle = thread::spawn(move || {
            eprintln!("[INFO] Storage pipeline started");
            
            while running.load(Ordering::Relaxed) {
                let start_time = std::time::Instant::now();
                
                // 1. 从共享缓冲区读取新数据，添加到队列
                if let Ok(buf) = buffer.read() {
                    let last_seq = storage_queue.last_stored_sequence();
                    let new_data = buf.get_history(config.batch_size)
                        .into_iter()
                        .filter(|d| d.sequence_number > last_seq)
                        .collect::<Vec<_>>();
                    
                    for data in new_data {
                        if let Err(e) = storage_queue.push(data) {
                            eprintln!("[ERROR] Failed to push to storage queue: {}", e);
                        }
                    }
                }
                
                // 2. 从队列取出数据批量存储
                let data_to_store = storage_queue.peek_batch(config.batch_size);
                
                if !data_to_store.is_empty() {
                    let max_sequence = data_to_store.iter()
                        .map(|d| d.sequence_number)
                        .max()
                        .unwrap_or(0);
                    
                    // 异步存储到数据库
                    let db_source_clone = Arc::clone(&db_source);
                    let storage_queue_clone = Arc::clone(&storage_queue);
                    let data_clone = data_to_store.clone();
                    let count = data_to_store.len();
                    
                    tokio_runtime.spawn(async move {
                        match db_source_clone.save_runtime_data_batch(&data_clone).await {
                            Ok(_) => {
                                // 存储成功，从队列删除
                                if let Err(e) = storage_queue_clone.remove_stored(count, max_sequence) {
                                    eprintln!("[ERROR] Failed to remove stored data: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Failed to save runtime data: {}", e);
                            }
                        }
                    });
                }
                
                // 3. 控制存储频率
                let elapsed = start_time.elapsed();
                if elapsed < config.interval {
                    thread::sleep(config.interval - elapsed);
                }
            }
            
            eprintln!("[INFO] Storage pipeline stopped");
        });
        
        self.handle = Some(handle);
    }
    
    /// 停止管道
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
    
    /// 异步回调：立即存储报警记录
    pub fn save_alarm_async(&self, data: ProcessedData) {
        let db_source = Arc::clone(&self.db_source);
        
        self.tokio_runtime.spawn(async move {
            if let Err(e) = db_source.save_alarm_record(&data).await {
                eprintln!("[ERROR] Failed to save alarm record: {}", e);
            }
        });
    }
}

impl Drop for StoragePipeline {
    fn drop(&mut self) {
        self.stop();
    }
}
```

### 5.6 报警回调机制

```rust
// src/pipeline/collection_pipeline.rs (修改)

impl CollectionPipeline {
    pub fn start(&mut self) {
        // ... 现有代码 ...
        
        let storage_pipeline = Arc::clone(&self.storage_pipeline);  // 新增：引用存储管道
        
        let handle = thread::spawn(move || {
            eprintln!("[INFO] Collection pipeline started");
            let mut consecutive_failures = 0;
            
            while running.load(Ordering::Relaxed) {
                let start_time = std::time::Instant::now();
                
                match Self::collect_with_retry(&repository, &config) {
                    Ok(sensor_data) => {
                        consecutive_failures = 0;
                        
                        let seq = sequence_number.fetch_add(1, Ordering::Relaxed);
                        let processed = ProcessedData::from_sensor_data(sensor_data, seq);
                        
                        // 检测报警状态，立即异步存储
                        if processed.is_danger {
                            eprintln!("[ALARM] Danger detected! Moment: {:.1}%", processed.moment_percentage);
                            storage_pipeline.save_alarm_async(processed.clone());
                        }
                        
                        // 写入共享缓冲区
                        if let Ok(mut buf) = buffer.write() {
                            buf.push(processed);
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        eprintln!("[ERROR] Collection failed: {} (consecutive: {})", 
                                  e, consecutive_failures);
                        
                        if let Ok(mut buf) = buffer.write() {
                            buf.record_error();
                        }
                        
                        if consecutive_failures >= config.disconnect_threshold {
                            eprintln!("[ERROR] Sensor disconnected (threshold reached)");
                        }
                    }
                }
                
                let elapsed = start_time.elapsed();
                if elapsed < config.interval {
                    thread::sleep(config.interval - elapsed);
                }
            }
            
            eprintln!("[INFO] Collection pipeline stopped");
        });
        
        self.handle = Some(handle);
    }
}
```

### 5.7 完整数据流

```
采集管道（100ms）
    ↓
检测到数据
    ├─ 正常数据 → 写入共享缓冲区 → 存储管道定时批量存储（1s）
    │                                    ↓
    │                              存储队列（避免重复）
    │                                    ↓
    │                              异步批量写入 runtime_data 表
    │                                    ↓
    │                              处理完删除队列数据
    │
    └─ 报警数据 → 立即异步回调 → 直接写入 alarm_records 表
                     ↓
                 不经过队列，立即存储
```

## 6. Pipeline 3: 显示管道

### 6.1 管道配置

```rust
// src/pipeline/display_pipeline.rs (新增)

use std::time::Duration;

/// 显示管道配置
#[derive(Debug, Clone)]
pub struct DisplayPipelineConfig {
    /// 刷新间隔
    pub interval: Duration,
    
    /// 是否启用节流
    pub enable_throttle: bool,
    
    /// 最小更新间隔（节流）
    pub min_update_interval: Duration,
}

impl Default for DisplayPipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),  // 100ms = 10Hz
            enable_throttle: true,
            min_update_interval: Duration::from_millis(100),
        }
    }
}
```

### 6.2 管道实现（Qt 主线程安全）

```rust
// src/pipeline/display_pipeline.rs (续)

use std::sync::Arc;
use std::time::Instant;
use crate::intents::monitoring_intent::MonitoringIntent;
use super::shared_buffer::SharedBuffer;

/// 显示管道（运行在 Qt 主线程）
pub struct DisplayPipeline {
    config: DisplayPipelineConfig,
    buffer: SharedBuffer,
    last_update_time: Instant,
    last_sequence: u64,
}

impl DisplayPipeline {
    pub fn new(config: DisplayPipelineConfig, buffer: SharedBuffer) -> Self {
        Self {
            config,
            buffer,
            last_update_time: Instant::now(),
            last_sequence: 0,
        }
    }
    
    /// 尝试更新 ViewModel（由 Qt Timer 调用）
    /// 返回 Option<Intent>，如果有新数据则返回 Intent
    pub fn try_update(&mut self) -> Option<MonitoringIntent> {
        // 节流检查
        if self.config.enable_throttle {
            let elapsed = self.last_update_time.elapsed();
            if elapsed < self.config.min_update_interval {
                return None;
            }
        }
        
        // 从共享缓冲区读取最新数据
        let processed_data = if let Ok(buf) = self.buffer.read() {
            buf.get_latest()
        } else {
            None
        };
        
        // 检查是否有新数据
        if let Some(data) = processed_data {
            if data.sequence_number > self.last_sequence {
                self.last_sequence = data.sequence_number;
                self.last_update_time = Instant::now();
                
                // 转换为 Intent
                return Some(MonitoringIntent::SensorDataUpdated(data.raw_data));
            }
        }
        
        None
    }
    
    /// 获取统计信息（用于调试）
    pub fn get_stats(&self) -> Option<super::shared_buffer::BufferStats> {
        if let Ok(buf) = self.buffer.read() {
            Some(buf.get_stats().clone())
        } else {
            None
        }
    }
}
```

### 6.3 Qt Timer 集成

```rust
// src/pipeline/display_timer.rs (新增)

use cxx_qt::CxxQtType;
use cxx_qt_lib::QTimer;
use std::pin::Pin;

/// 显示管道定时器（QML 单例）
#[cxx_qt::bridge]
pub mod display_timer_bridge {
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        type DisplayTimer = super::DisplayTimerRust;
        
        /// 启动定时器
        #[qinvokable]
        unsafe fn start_timer(self: Pin<&mut DisplayTimer>, interval_ms: i32);
        
        /// 停止定时器
        #[qinvokable]
        unsafe fn stop_timer(self: Pin<&mut DisplayTimer>);
    }
}

use std::sync::Arc;
use super::display_pipeline::DisplayPipeline;
use super::shared_buffer::SharedBuffer;

pub struct DisplayTimerRust {
    pipeline: Option<DisplayPipeline>,
    timer: Option<QTimer>,
}

impl Default for DisplayTimerRust {
    fn default() -> Self {
        Self {
            pipeline: None,
            timer: None,
        }
    }
}

impl display_timer_bridge::DisplayTimer {
    /// 初始化管道
    pub fn init_pipeline(&mut self, buffer: SharedBuffer) {
        let config = super::display_pipeline::DisplayPipelineConfig::default();
        self.pipeline = Some(DisplayPipeline::new(config, buffer));
    }
    
    /// 启动定时器
    pub fn start_timer(mut self: Pin<&mut Self>, interval_ms: i32) {
        // TODO: 创建 QTimer 并连接到 tick 槽
        eprintln!("[INFO] Display timer started: {}ms", interval_ms);
    }
    
    /// 停止定时器
    pub fn stop_timer(mut self: Pin<&mut Self>) {
        eprintln!("[INFO] Display timer stopped");
    }
    
    /// 定时器回调（由 Qt Timer 触发）
    fn on_timer_tick(mut self: Pin<&mut Self>) {
        if let Some(pipeline) = &mut self.pipeline {
            if let Some(intent) = pipeline.try_update() {
                // TODO: 获取 ViewModel 引用并调用 handle_intent
                eprintln!("[DEBUG] Display update: {:?}", intent);
            }
        }
    }
}
```

## 7. 管道管理器（改进版）

### 7.1 统一管理三个管道

```rust
// src/pipeline/pipeline_manager.rs (新增)

use std::sync::Arc;
use crate::repositories::CraneDataRepository;
use super::shared_buffer::{ProcessedDataBuffer, SharedBuffer};
use super::collection_pipeline::{CollectionPipeline, CollectionPipelineConfig};
use super::storage_pipeline::{StoragePipeline, StoragePipelineConfig};
use super::display_pipeline::{DisplayPipeline, DisplayPipelineConfig};

/// 管道管理器
pub struct PipelineManager {
    collection_pipeline: CollectionPipeline,
    storage_pipeline: Arc<StoragePipeline>,  // Arc 包装，供采集管道使用
    display_pipeline: DisplayPipeline,
    shared_buffer: SharedBuffer,
}

impl PipelineManager {
    /// 创建管道管理器
    pub fn new(repository: Arc<CraneDataRepository>, db_path: &str) -> Result<Self, String> {
        // 创建共享缓冲区
        let shared_buffer = Arc::new(std::sync::RwLock::new(
            ProcessedDataBuffer::new(1000)  // 保留最近 1000 条数据
        ));
        
        // 创建存储管道（先创建，供采集管道引用）
        let storage_config = StoragePipelineConfig::default();
        let storage_pipeline = Arc::new(StoragePipeline::new(
            storage_config,
            db_path,
            Arc::clone(&shared_buffer),
        )?);
        
        // 创建采集管道（引用存储管道）
        let collection_config = CollectionPipelineConfig::default();
        let collection_pipeline = CollectionPipeline::new(
            collection_config,
            Arc::clone(&repository),
            Arc::clone(&shared_buffer),
            Arc::clone(&storage_pipeline),  // 传递存储管道引用
        );
        
        // 创建显示管道
        let display_config = DisplayPipelineConfig::default();
        let display_pipeline = DisplayPipeline::new(
            display_config,
            Arc::clone(&shared_buffer),
        );
        
        Ok(Self {
            collection_pipeline,
            storage_pipeline,
            display_pipeline,
            shared_buffer,
        })
    }
    
    /// 启动所有管道
    pub fn start_all(&mut self) {
        eprintln!("[INFO] Starting all pipelines...");
        
        // 先启动存储管道
        Arc::get_mut(&mut self.storage_pipeline)
            .expect("Storage pipeline should be exclusively owned")
            .start();
        
        // 再启动采集管道
        self.collection_pipeline.start();
        
        eprintln!("[INFO] All pipelines started (display pipeline runs on Qt timer)");
    }
    
    /// 停止所有管道
    pub fn stop_all(&mut self) {
        eprintln!("[INFO] Stopping all pipelines...");
        
        // 先停止采集管道
        self.collection_pipeline.stop();
        
        // 等待存储队列清空
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        // 再停止存储管道
        Arc::get_mut(&mut self.storage_pipeline)
            .expect("Storage pipeline should be exclusively owned")
            .stop();
        
        eprintln!("[INFO] All pipelines stopped");
    }
    
    /// 获取显示管道引用（用于 Qt Timer）
    pub fn get_display_pipeline(&mut self) -> &mut DisplayPipeline {
        &mut self.display_pipeline
    }
    
    /// 获取共享缓冲区（用于调试）
    pub fn get_shared_buffer(&self) -> SharedBuffer {
        Arc::clone(&self.shared_buffer)
    }
    
    /// 获取存储队列状态
    pub fn get_storage_queue_status(&self) -> (usize, u64) {
        // 返回 (队列长度, 最后存储序列号)
        // TODO: 实现获取存储队列状态的方法
        (0, 0)
    }
}

impl Drop for PipelineManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}
```

### 7.2 Cargo.toml 依赖更新

```toml
[dependencies]
# ... 现有依赖 ...

# SQLite 数据库
rusqlite = { version = "0.31", features = ["bundled"] }

# 异步运行时
tokio = { version = "1.42", features = ["full"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
```

### 7.3 模块结构

```
src/pipeline/
├── mod.rs                      # 模块导出
├── shared_buffer.rs            # 共享缓冲区
├── storage_queue.rs            # 存储队列（新增）
├── collection_pipeline.rs      # 采集管道
├── storage_pipeline.rs         # 存储管道（改进）
├── display_pipeline.rs         # 显示管道
└── pipeline_manager.rs         # 管道管理器

src/data_sources/
├── mod.rs
├── sensor_data_source.rs
├── config_data_source.rs
└── sqlite_data_source.rs       # SQLite 数据源（新增）
```

## 8. 使用示例

### 8.1 初始化和启动

```rust
// src/application.rs 或 src/main.rs

use crate::pipeline::pipeline_manager::PipelineManager;
use crate::repositories::CraneDataRepository;
use std::sync::Arc;

fn main() {
    // 创建数据仓库
    let repository = Arc::new(CraneDataRepository::new());
    
    // 创建管道管理器（指定数据库路径）
    let mut pipeline_manager = PipelineManager::new(
        repository,
        "data/crane_monitor.db"  // SQLite 数据库路径
    ).expect("Failed to create pipeline manager");
    
    // 启动所有管道
    pipeline_manager.start_all();
    
    // 应用运行...
    
    // 停止管道（应用退出时）
    pipeline_manager.stop_all();
}
```

### 8.2 查询数据库

```rust
// 查询最近的运行数据
pub fn query_recent_runtime_data(db_path: &str, limit: usize) -> Result<Vec<RuntimeData>, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT * FROM runtime_data 
         ORDER BY timestamp DESC 
         LIMIT ?1"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let rows = stmt.query_map([limit], |row| {
        Ok(RuntimeData {
            id: row.get(0)?,
            sequence_number: row.get(1)?,
            timestamp: row.get(2)?,
            current_load: row.get(3)?,
            rated_load: row.get(4)?,
            working_radius: row.get(5)?,
            boom_angle: row.get(6)?,
            boom_length: row.get(7)?,
            moment_percentage: row.get(8)?,
            is_danger: row.get(9)?,
            validation_error: row.get(10)?,
        })
    }).map_err(|e| format!("Failed to query: {}", e))?;
    
    let mut data = Vec::new();
    for row in rows {
        data.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
    }
    
    Ok(data)
}

// 查询未确认的报警记录
pub fn query_unacknowledged_alarms(db_path: &str) -> Result<Vec<AlarmRecord>, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut stmt = conn.prepare(
        "SELECT * FROM alarm_records 
         WHERE acknowledged = 0 
         ORDER BY timestamp DESC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let rows = stmt.query_map([], |row| {
        Ok(AlarmRecord {
            id: row.get(0)?,
            sequence_number: row.get(1)?,
            timestamp: row.get(2)?,
            alarm_type: row.get(3)?,
            current_load: row.get(4)?,
            rated_load: row.get(5)?,
            working_radius: row.get(6)?,
            boom_angle: row.get(7)?,
            boom_length: row.get(8)?,
            moment_percentage: row.get(9)?,
            description: row.get(10)?,
            acknowledged: row.get(11)?,
            acknowledged_at: row.get(12)?,
        })
    }).map_err(|e| format!("Failed to query: {}", e))?;
    
    let mut alarms = Vec::new();
    for row in rows {
        alarms.push(row.map_err(|e| format!("Failed to parse row: {}", e))?);
    }
    
    Ok(alarms)
}

// 确认报警
pub fn acknowledge_alarm(db_path: &str, alarm_id: i64) -> Result<(), String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    conn.execute(
        "UPDATE alarm_records 
         SET acknowledged = 1, acknowledged_at = strftime('%s', 'now')
         WHERE id = ?1",
        params![alarm_id],
    ).map_err(|e| format!("Failed to update: {}", e))?;
    
    Ok(())
}
```

### 8.3 监控管道状态

```rust
// 获取管道健康状态
pub fn check_pipeline_health(pipeline_manager: &PipelineManager) {
    let buffer = pipeline_manager.get_shared_buffer();
    
    if let Ok(buf) = buffer.read() {
        let stats = buf.get_stats();
        
        println!("=== Pipeline Health ===");
        println!("Total collections: {}", stats.total_collections);
        println!("Success count: {}", stats.success_count);
        println!("Error count: {}", stats.error_count);
        println!("Success rate: {:.1}%", 
                 (stats.success_count as f64 / stats.total_collections as f64) * 100.0);
        
        if let Some(last_update) = stats.last_update_time {
            if let Ok(elapsed) = std::time::SystemTime::now().duration_since(last_update) {
                println!("Last update: {:?} ago", elapsed);
                
                if elapsed > std::time::Duration::from_secs(2) {
                    println!("⚠️  WARNING: Data is stale!");
                }
            }
        }
    }
    
    let (queue_len, last_seq) = pipeline_manager.get_storage_queue_status();
    println!("Storage queue length: {}", queue_len);
    println!("Last stored sequence: {}", last_seq);
}
```

## 9. 与现有 MVI 架构集成

### 8.1 更新 ViewModelManager

```rust
// src/viewmodel_manager.rs (修改)

use crate::pipeline::pipeline_manager::PipelineManager;
use crate::repositories::CraneDataRepository;
use std::sync::{Arc, Mutex};

pub struct ViewModelManager {
    /// 管道管理器
    pipeline_manager: Option<PipelineManager>,
    
    /// ViewModel 是否已准备好
    viewmodel_ready: bool,
}

impl ViewModelManager {
    pub fn new() -> Self {
        Self {
            pipeline_manager: None,
            viewmodel_ready: false,
        }
    }
    
    pub fn mark_viewmodel_ready(&mut self) {
        eprintln!("[INFO] ViewModel marked as ready");
        self.viewmodel_ready = true;
    }
    
    /// 启动三管道数据采集
    pub fn start_data_collection(&mut self) {
        if !self.viewmodel_ready {
            eprintln!("[WARN] ViewModel not ready, cannot start data collection");
            return;
        }
        
        eprintln!("[INFO] Starting three-pipeline data collection...");
        
        // 创建数据仓库
        let repository = Arc::new(CraneDataRepository::new());
        
        // 创建管道管理器
        let mut manager = PipelineManager::new(repository);
        
        // 启动所有管道
        manager.start_all();
        
        self.pipeline_manager = Some(manager);
        eprintln!("[INFO] Three-pipeline data collection started");
    }
    
    /// 停止数据采集
    pub fn stop_data_collection(&mut self) {
        if let Some(mut manager) = self.pipeline_manager.take() {
            manager.stop_all();
            eprintln!("[INFO] Data collection stopped");
        }
    }
}
```

### 8.2 Intent 保持不变

现有的 `MonitoringIntent` 无需修改，显示管道会生成相同的 Intent：

```rust
// src/intents/monitoring_intent.rs (无需修改)

#[derive(Debug, Clone)]
pub enum MonitoringIntent {
    ClearError,
    ResetAlarm,
    SensorDataUpdated(SensorData),  // 显示管道生成此 Intent
    SensorDisconnected,
    SensorReconnected,
}
```

### 8.3 Reducer 保持不变

现有的 `MonitoringReducer` 无需修改，继续处理 Intent：

```rust
// src/reducers/monitoring_reducer.rs (无需修改)
// Reducer 逻辑保持不变
```

## 9. 配置管理

### 9.1 管道配置文件

```yaml
# config/pipeline_config.yaml

collection:
  interval_ms: 100          # 采集间隔 100ms
  max_retries: 3            # 最大重试次数
  retry_delay_ms: 10        # 重试延迟
  disconnect_threshold: 10  # 断连检测阈值

storage:
  interval_ms: 1000         # 存储间隔 1秒
  batch_size: 10            # 批量存储大小
  max_retries: 3            # 最大重试次数
  retry_delay_ms: 100       # 重试延迟

display:
  interval_ms: 100          # 刷新间隔 100ms
  enable_throttle: true     # 启用节流
  min_update_interval_ms: 100  # 最小更新间隔
```

### 9.2 配置加载

```rust
// src/pipeline/config.rs (新增)

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfigFile {
    pub collection: CollectionConfig,
    pub storage: StorageConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub interval_ms: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub disconnect_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub interval_ms: u64,
    pub batch_size: usize,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub interval_ms: u64,
    pub enable_throttle: bool,
    pub min_update_interval_ms: u64,
}

impl PipelineConfigFile {
    /// 从文件加载配置
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        // TODO: 实现 YAML 文件读取
        Ok(Self::default())
    }
    
    /// 转换为运行时配置
    pub fn to_runtime_configs(&self) -> (
        super::collection_pipeline::CollectionPipelineConfig,
        super::storage_pipeline::StoragePipelineConfig,
        super::display_pipeline::DisplayPipelineConfig,
    ) {
        let collection = super::collection_pipeline::CollectionPipelineConfig {
            interval: Duration::from_millis(self.collection.interval_ms),
            max_retries: self.collection.max_retries,
            retry_delay: Duration::from_millis(self.collection.retry_delay_ms),
            disconnect_threshold: self.collection.disconnect_threshold,
        };
        
        let storage = super::storage_pipeline::StoragePipelineConfig {
            interval: Duration::from_millis(self.storage.interval_ms),
            batch_size: self.storage.batch_size,
            max_retries: self.storage.max_retries,
            retry_delay: Duration::from_millis(self.storage.retry_delay_ms),
        };
        
        let display = super::display_pipeline::DisplayPipelineConfig {
            interval: Duration::from_millis(self.display.interval_ms),
            enable_throttle: self.display.enable_throttle,
            min_update_interval: Duration::from_millis(self.display.min_update_interval_ms),
        };
        
        (collection, storage, display)
    }
}

impl Default for PipelineConfigFile {
    fn default() -> Self {
        Self {
            collection: CollectionConfig {
                interval_ms: 100,
                max_retries: 3,
                retry_delay_ms: 10,
                disconnect_threshold: 10,
            },
            storage: StorageConfig {
                interval_ms: 1000,
                batch_size: 10,
                max_retries: 3,
                retry_delay_ms: 100,
            },
            display: DisplayConfig {
                interval_ms: 100,
                enable_throttle: true,
                min_update_interval_ms: 100,
            },
        }
    }
}
```

## 10. 错误处理和恢复

### 10.1 采集管道错误处理

```rust
// 采集管道错误处理策略

1. 单次采集失败
   - 重试 3 次（可配置）
   - 每次重试延迟 10ms
   - 记录错误到共享缓冲区统计

2. 连续失败检测
   - 连续失败 10 次（可配置）触发断连事件
   - 生成 SensorDisconnected Intent
   - UI 显示传感器断连提示

3. 数据验证失败
   - 记录验证错误到 ProcessedData
   - 继续处理，不中断管道
   - UI 显示数据异常提示

4. 降级策略
   - 使用上一次有效数据
   - 标记数据为"过期"
   - UI 显示数据过期警告
```

### 10.2 存储管道错误处理

```rust
// 存储管道错误处理策略

1. 存储失败
   - 重试 3 次（可配置）
   - 每次重试延迟 100ms
   - 失败后记录到日志文件

2. 数据库锁定
   - 等待并重试
   - 超时后跳过本次存储
   - 不影响下次存储

3. 磁盘空间不足
   - 停止存储管道
   - 生成系统错误事件
   - UI 显示磁盘空间警告

4. 降级策略
   - 只存储关键数据（报警记录）
   - 跳过普通数据存储
   - 恢复后自动恢复正常存储
```

### 10.3 显示管道错误处理

```rust
// 显示管道错误处理策略

1. 读取缓冲区失败
   - 使用上一次缓存数据
   - 继续尝试读取
   - 不中断 UI 刷新

2. Intent 处理失败
   - 记录错误日志
   - 跳过本次更新
   - 不影响下次更新

3. 数据过期检测
   - 检查数据时间戳
   - 超过 1 秒标记为过期
   - UI 显示数据过期提示

4. 降级策略
   - 降低刷新频率（100ms → 500ms）
   - 只更新关键数据
   - 恢复后自动恢复正常频率
```

## 11. 监控和诊断

### 11.1 性能指标

```rust
// src/pipeline/metrics.rs (新增)

use std::time::{Duration, SystemTime};

/// 管道性能指标
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    /// 管道名称
    pub name: String,
    
    /// 总执行次数
    pub total_executions: u64,
    
    /// 成功次数
    pub success_count: u64,
    
    /// 失败次数
    pub error_count: u64,
    
    /// 平均执行时间
    pub avg_execution_time: Duration,
    
    /// 最大执行时间
    pub max_execution_time: Duration,
    
    /// 最小执行时间
    pub min_execution_time: Duration,
    
    /// 最后执行时间
    pub last_execution_time: Option<SystemTime>,
}

impl PipelineMetrics {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            total_executions: 0,
            success_count: 0,
            error_count: 0,
            avg_execution_time: Duration::ZERO,
            max_execution_time: Duration::ZERO,
            min_execution_time: Duration::MAX,
            last_execution_time: None,
        }
    }
    
    /// 记录成功执行
    pub fn record_success(&mut self, duration: Duration) {
        self.total_executions += 1;
        self.success_count += 1;
        self.update_timing(duration);
    }
    
    /// 记录失败执行
    pub fn record_error(&mut self, duration: Duration) {
        self.total_executions += 1;
        self.error_count += 1;
        self.update_timing(duration);
    }
    
    fn update_timing(&mut self, duration: Duration) {
        // 更新平均时间
        let total_time = self.avg_execution_time * (self.total_executions - 1) as u32;
        self.avg_execution_time = (total_time + duration) / self.total_executions as u32;
        
        // 更新最大/最小时间
        if duration > self.max_execution_time {
            self.max_execution_time = duration;
        }
        if duration < self.min_execution_time {
            self.min_execution_time = duration;
        }
        
        self.last_execution_time = Some(SystemTime::now());
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.success_count as f64 / self.total_executions as f64) * 100.0
        }
    }
}
```

### 11.2 健康检查

```rust
// src/pipeline/health_check.rs (新增)

use std::time::{Duration, SystemTime};

/// 管道健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineHealth {
    /// 健康
    Healthy,
    /// 警告（性能下降）
    Warning(String),
    /// 错误（功能异常）
    Error(String),
    /// 停止
    Stopped,
}

/// 健康检查器
pub struct HealthChecker {
    /// 最大允许失败率（百分比）
    max_error_rate: f64,
    
    /// 最大允许延迟
    max_latency: Duration,
    
    /// 数据过期阈值
    data_expiry_threshold: Duration,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self {
            max_error_rate: 10.0,  // 10% 失败率
            max_latency: Duration::from_millis(500),
            data_expiry_threshold: Duration::from_secs(2),
        }
    }
}

impl HealthChecker {
    /// 检查管道健康状态
    pub fn check_pipeline(
        &self,
        metrics: &super::metrics::PipelineMetrics,
    ) -> PipelineHealth {
        // 检查是否有执行记录
        if metrics.total_executions == 0 {
            return PipelineHealth::Stopped;
        }
        
        // 检查失败率
        let error_rate = (metrics.error_count as f64 / metrics.total_executions as f64) * 100.0;
        if error_rate > self.max_error_rate {
            return PipelineHealth::Error(
                format!("High error rate: {:.1}%", error_rate)
            );
        }
        
        // 检查延迟
        if metrics.avg_execution_time > self.max_latency {
            return PipelineHealth::Warning(
                format!("High latency: {:?}", metrics.avg_execution_time)
            );
        }
        
        // 检查数据新鲜度
        if let Some(last_time) = metrics.last_execution_time {
            if let Ok(elapsed) = SystemTime::now().duration_since(last_time) {
                if elapsed > self.data_expiry_threshold {
                    return PipelineHealth::Warning(
                        format!("Data expired: {:?} ago", elapsed)
                    );
                }
            }
        }
        
        PipelineHealth::Healthy
    }
}
```

## 12. 迁移计划

### 12.1 从现有架构迁移

```
阶段 1: 准备（1-2天）
  ✓ 创建新的数据模型 (ProcessedData)
  ✓ 创建共享缓冲区 (SharedBuffer)
  ✓ 创建管道配置结构

阶段 2: 实现管道（3-5天）
  ✓ 实现采集与处理管道
  ✓ 实现存储管道
  ✓ 实现显示管道
  ✓ 实现管道管理器

阶段 3: 集成测试（2-3天）
  ✓ 单元测试各个管道
  ✓ 集成测试三管道协作
  ✓ 性能测试和优化

阶段 4: 替换现有实现（1-2天）
  ✓ 更新 ViewModelManager
  ✓ 移除旧的 DataCollector
  ✓ 更新 QML 集成

阶段 5: 验证和部署（1-2天）
  ✓ 功能验证
  ✓ 性能验证
  ✓ 部署到目标设备
```

### 12.2 兼容性保证

```rust
// 保持现有接口不变

1. Intent 定义不变
   - MonitoringIntent 保持原样
   - Reducer 逻辑不变

2. ViewModel 接口不变
   - handle_intent() 方法保持原样
   - Qt 属性定义不变

3. QML 代码不变
   - 属性绑定不变
   - 方法调用不变

4. 数据模型兼容
   - SensorData 保持原样
   - ProcessedData 作为内部类型
```

## 13. 测试策略

### 13.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shared_buffer() {
        let mut buffer = ProcessedDataBuffer::new(10);
        
        // 测试写入
        let data = ProcessedData::from_sensor_data(
            SensorData::new(10.0, 5.0, 45.0),
            1
        );
        buffer.push(data.clone());
        
        // 测试读取
        assert_eq!(buffer.get_latest().unwrap().sequence_number, 1);
        
        // 测试历史
        assert_eq!(buffer.get_history(5).len(), 1);
    }
    
    #[test]
    fn test_processed_data_calculation() {
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        
        // 验证力矩计算
        assert_eq!(processed.moment_percentage, 80.0);
        
        // 验证危险状态
        assert!(!processed.is_danger);
    }
}
```

### 13.2 集成测试

```rust
#[test]
fn test_three_pipeline_integration() {
    // 创建管道管理器
    let repository = Arc::new(CraneDataRepository::new());
    let mut manager = PipelineManager::new(repository);
    
    // 启动管道
    manager.start_all();
    
    // 等待数据采集
    std::thread::sleep(Duration::from_millis(500));
    
    // 验证共享缓冲区有数据
    let buffer = manager.get_shared_buffer();
    let has_data = buffer.read().unwrap().get_latest().is_some();
    assert!(has_data);
    
    // 停止管道
    manager.stop_all();
}
```

## 14. 性能优化建议

### 14.1 内存优化

```rust
1. 限制历史数据大小
   - 默认保留 1000 条记录
   - 可配置最大容量
   - 自动清理旧数据

2. 使用 Arc 共享数据
   - 避免数据复制
   - 减少内存占用

3. 及时释放资源
   - 管道停止时清理缓冲区
   - 使用 Drop trait 自动清理
```

### 14.2 CPU 优化

```rust
1. 避免频繁锁竞争
   - 读写锁分离
   - 减少锁持有时间
   - 使用原子操作

2. 批量处理
   - 存储管道批量写入
   - 减少 I/O 次数

3. 节流机制
   - 显示管道节流
   - 避免过度刷新
```

## 15. 总结

### 15.1 架构优势

✅ **完全解耦**: 三个管道独立运行，互不影响
✅ **独立频率**: 采集 100ms、存储 1s、显示 100ms 可独立配置
✅ **容错性强**: 单个管道故障不影响其他管道
✅ **线程安全**: 使用 Rust 并发原语保证安全
✅ **MVI 兼容**: 与现有架构无缝集成
✅ **易于测试**: 每个管道可独立测试
✅ **易于监控**: 完善的指标和健康检查
✅ **易于扩展**: 可轻松添加新管道

### 15.2 后台线程 2（存储管道）核心特性（解耦版）

#### ✅ 数据库操作解耦
- **抽象接口**: `StorageRepository` trait 定义存储操作
- **依赖注入**: 存储管道依赖抽象接口，而非具体实现
- **易于测试**: 可以使用 `MockStorageRepository` 进行单元测试
- **支持多种数据库**: SQLite、PostgreSQL、MySQL 等

#### ✅ 双表设计
- **runtime_data 表**: 存储正常运行数据，定时批量写入
- **alarm_records 表**: 存储报警信息，异步回调立即写入

#### ✅ 避免重复存储
- 使用 `sequence_number` 唯一标识每条数据
- 存储队列追踪 `last_stored_sequence`
- 数据库表使用 `UNIQUE` 约束防止重复

#### ✅ 管道队列机制
```
共享缓冲区 → 存储队列 → 异步批量存储 → 删除已存储数据
     ↓
  避免重复
```

#### ✅ 异步回调存储报警
```
采集管道检测到报警 → 立即调用 save_alarm_async()
                          ↓
                    Tokio 异步任务
                          ↓
                    repository.save_alarm_record()（抽象接口）
                          ↓
                    SqliteStorageRepository 实现
                          ↓
                    直接写入 alarm_records 表
```

#### ✅ 事务批处理
- 使用 SQLite 事务批量插入
- 提高存储效率，减少磁盘 I/O
- 失败自动回滚，保证数据一致性

### 15.3 关键技术点（解耦版）

1. **存储队列**: `StorageQueue` 管理待存储数据，避免重复
2. **序列号追踪**: `last_stored_sequence` 记录已存储位置
3. **异步存储**: Tokio runtime 处理数据库操作
4. **双路存储**: 
   - 正常数据：队列 → 批量存储
   - 报警数据：回调 → 立即存储
5. **事务保护**: SQLite 事务保证批量插入的原子性
6. **依赖抽象**: `StorageRepository` trait 解耦数据库操作
7. **依赖注入**: 通过构造函数注入具体实现
8. **易于测试**: 使用 `MockStorageRepository` 进行单元测试

### 15.4 数据流对比

**改进前（有重复存储问题）**:
```
共享缓冲区 → 每次读取最近 10 条 → 存储
                ↓
            可能重复存储相同数据
```

**改进后（无重复）**:
```
共享缓冲区 → 过滤已存储数据 → 存储队列 → 批量存储 → 删除队列数据
                ↓                                    ↓
          使用 sequence_number              更新 last_stored_sequence
```

### 15.5 性能优化

| 优化项 | 方法 | 效果 |
|--------|------|------|
| 避免重复存储 | 序列号追踪 | 减少无效写入 |
| 批量写入 | SQLite 事务 | 提升 10-100 倍性能 |
| 异步处理 | Tokio runtime | 不阻塞主线程 |
| 队列管理 | 处理完删除 | 控制内存占用 |
| 报警优先 | 异步回调 | 实时响应 |

### 15.6 下一步工作

**必须实现**:
- [x] 设计双表结构（runtime_data + alarm_records）
- [x] 实现 StorageQueue（避免重复存储）
- [x] 设计 StorageRepository trait（数据库操作解耦）
- [x] 实现 SqliteStorageRepository（SQLite 实现）
- [x] 实现 MockStorageRepository（测试用）
- [x] 实现异步报警回调机制
- [ ] 实现 CollectionPipeline 引用 StoragePipeline
- [ ] 编写单元测试
- [ ] 编写集成测试

**可选优化**:
- [ ] 数据库自动清理（删除过期数据）
- [ ] 存储失败重试队列
- [ ] 数据压缩存储
- [ ] 数据导出功能
- [ ] PostgreSQL 实现
- [ ] MySQL 实现

### 15.7 配置示例

```yaml
# config/pipeline_config.yaml

storage:
  # 运行数据存储配置
  interval_ms: 1000           # 1秒批量存储一次
  batch_size: 10              # 每次最多存储 10 条
  max_queue_size: 1000        # 队列最大容量
  
  # 数据库配置
  db_path: "data/crane_monitor.db"
  
  # 重试配置
  max_retries: 3
  retry_delay_ms: 100
  
  # 报警配置
  alarm_immediate: true       # 报警立即存储
  alarm_async: true           # 使用异步回调
```

---

**文档版本**: 3.0（解耦版本）
**创建日期**: 2026-03-19  
**更新日期**: 2026-03-20
**作者**: Crane 开发团队  
**状态**: 设计完成，待实施

**主要改进**:
1. ✅ 双表设计（运行数据 + 报警信息）
2. ✅ 避免重复存储（序列号追踪 + 队列管理）
3. ✅ 异步报警回调（立即存储）
4. ✅ 事务批处理（提升性能）
5. ✅ 数据库操作解耦（StorageRepository trait）
6. ✅ 依赖注入（易于测试和扩展）
7. ✅ 完整的代码示例和使用指南

**相关文档**:
- `doc/STORAGE_DECOUPLING_DESIGN.md` - 存储解耦详细设计
- `doc/STORAGE_PIPELINE_DESIGN.md` - 存储管道快速参考
