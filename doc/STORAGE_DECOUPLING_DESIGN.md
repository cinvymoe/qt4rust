# 存储管道解耦设计

## 1. 设计原则

将数据库操作从存储管道中解耦，使用 trait 抽象和 Repository 模式。

### 优势
- ✅ 易于测试（可以 mock Repository）
- ✅ 支持多种数据库（SQLite、PostgreSQL、MySQL）
- ✅ 符合依赖倒置原则（依赖抽象而非具体实现）
- ✅ 业务逻辑与数据访问分离

## 2. 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│              StoragePipeline（管道逻辑层）                   │
│  - 队列管理                                                  │
│  - 定时调度                                                  │
│  - 异步任务编排                                              │
│  - 依赖 StorageRepository trait（抽象）                     │
└─────────────────────────────────────────────────────────────┘
                        ↓ 依赖抽象接口
┌─────────────────────────────────────────────────────────────┐
│          StorageRepository trait（存储接口层）               │
│  - save_runtime_data_batch()                                │
│  - save_alarm_record()                                      │
│  - query_recent_runtime_data()                              │
│  - query_unacknowledged_alarms()                            │
└─────────────────────────────────────────────────────────────┘
                        ↓ 具体实现
┌─────────────────────────────────────────────────────────────┐
│      SqliteStorageRepository（SQLite 实现层）                │
│  - 实现 StorageRepository trait                             │
│  - 事务管理                                                  │
│  - 错误处理                                                  │
│  - 使用 SqliteDataSource                                    │
└─────────────────────────────────────────────────────────────┘
                        ↓ 使用
┌─────────────────────────────────────────────────────────────┐
│         SqliteDataSource（数据库操作层）                     │
│  - 连接管理                                                  │
│  - SQL 执行                                                  │
│  - 结果映射                                                  │
└─────────────────────────────────────────────────────────────┘
```

## 3. 核心 trait 定义

```rust
// src/repositories/storage_repository.rs

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

## 4. SQLite 实现

```rust
// src/repositories/sqlite_storage_repository.rs

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::Connection;
use crate::repositories::storage_repository::StorageRepository;
use crate::models::processed_data::ProcessedData;
use crate::models::alarm_record::AlarmRecord;

/// SQLite 存储仓库实现
pub struct SqliteStorageRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteStorageRepository {
    /// 创建新的 SQLite 存储仓库
    pub async fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let repo = Self {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        // 初始化表
        repo.init_tables().await?;
        
        Ok(repo)
    }
    
    /// 初始化数据库表
    async fn init_tables(&self) -> Result<(), String> {
        let conn = self.connection.lock().await;
        
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
}

#[async_trait]
impl StorageRepository for SqliteStorageRepository {
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String> {
        if data.is_empty() {
            return Ok(0);
        }
        
        let conn = self.connection.lock().await;
        
        // 开始事务
        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;
        
        let mut saved_count = 0;
        
        for item in data {
            let timestamp = item.timestamp.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            
            let result = conn.execute(
                "INSERT OR IGNORE INTO runtime_data 
                 (sequence_number, timestamp, current_load, rated_load, working_radius, 
                  boom_angle, boom_length, moment_percentage, is_danger, validation_error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
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
            
            match result {
                Ok(rows) => saved_count += rows,
                Err(e) => {
                    // 回滚事务
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(format!("Failed to insert runtime data: {}", e));
                }
            }
        }
        
        // 提交事务
        conn.execute("COMMIT", [])
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
        
        eprintln!("[INFO] Saved {} runtime records to database", saved_count);
        Ok(saved_count)
    }
    
    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String> {
        let conn = self.connection.lock().await;
        
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
            rusqlite::params![
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
        
        let alarm_id = conn.last_insert_rowid();
        
        eprintln!("[INFO] Saved alarm record: {} (id: {})", alarm_type, alarm_id);
        Ok(alarm_id)
    }
    
    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String> {
        // TODO: 实现查询逻辑
        Ok(Vec::new())
    }
    
    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String> {
        // TODO: 实现查询逻辑
        Ok(Vec::new())
    }
    
    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String> {
        let conn = self.connection.lock().await;
        
        conn.execute(
            "UPDATE alarm_records 
             SET acknowledged = 1, acknowledged_at = strftime('%s', 'now')
             WHERE id = ?1",
            rusqlite::params![alarm_id],
        ).map_err(|e| format!("Failed to acknowledge alarm: {}", e))?;
        
        Ok(())
    }
    
    async fn get_last_stored_sequence(&self) -> Result<u64, String> {
        let conn = self.connection.lock().await;
        
        let result: Result<i64, _> = conn.query_row(
            "SELECT MAX(sequence_number) FROM runtime_data",
            [],
            |row| row.get(0),
        );
        
        match result {
            Ok(seq) => Ok(seq as u64),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(format!("Failed to get last sequence: {}", e)),
        }
    }
    
    async fn health_check(&self) -> Result<(), String> {
        let conn = self.connection.lock().await;
        
        conn.execute("SELECT 1", [])
            .map_err(|e| format!("Health check failed: {}", e))?;
        
        Ok(())
    }
}
```

继续下一部分...


## 5. 存储管道（解耦后）

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
    repository: Arc<dyn StorageRepository>,  // 依赖抽象接口
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
        let repository = Arc::clone(&self.repository);
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

## 6. Mock 实现（用于测试）

```rust
// src/repositories/mock_storage_repository.rs

use async_trait::async_trait;
use std::sync::Mutex;
use crate::repositories::storage_repository::StorageRepository;
use crate::models::processed_data::ProcessedData;
use crate::models::alarm_record::AlarmRecord;

/// Mock 存储仓库（用于测试）
pub struct MockStorageRepository {
    runtime_data: Mutex<Vec<ProcessedData>>,
    alarm_records: Mutex<Vec<AlarmRecord>>,
    should_fail: Mutex<bool>,
}

impl MockStorageRepository {
    pub fn new() -> Self {
        Self {
            runtime_data: Mutex::new(Vec::new()),
            alarm_records: Mutex::new(Vec::new()),
            should_fail: Mutex::new(false),
        }
    }
    
    /// 设置是否模拟失败
    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }
    
    /// 获取存储的运行数据数量
    pub fn get_runtime_data_count(&self) -> usize {
        self.runtime_data.lock().unwrap().len()
    }
    
    /// 获取存储的报警数量
    pub fn get_alarm_count(&self) -> usize {
        self.alarm_records.lock().unwrap().len()
    }
}

#[async_trait]
impl StorageRepository for MockStorageRepository {
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String> {
        if *self.should_fail.lock().unwrap() {
            return Err("Mock failure".to_string());
        }
        
        let mut storage = self.runtime_data.lock().unwrap();
        storage.extend_from_slice(data);
        Ok(data.len())
    }
    
    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String> {
        if *self.should_fail.lock().unwrap() {
            return Err("Mock failure".to_string());
        }
        
        let mut alarms = self.alarm_records.lock().unwrap();
        let alarm = AlarmRecord::from_processed_data(data);
        alarms.push(alarm);
        Ok(alarms.len() as i64)
    }
    
    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String> {
        let storage = self.runtime_data.lock().unwrap();
        Ok(storage.iter().rev().take(limit).cloned().collect())
    }
    
    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String> {
        let alarms = self.alarm_records.lock().unwrap();
        Ok(alarms.iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect())
    }
    
    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String> {
        let mut alarms = self.alarm_records.lock().unwrap();
        if let Some(alarm) = alarms.get_mut((alarm_id - 1) as usize) {
            alarm.acknowledged = true;
            Ok(())
        } else {
            Err("Alarm not found".to_string())
        }
    }
    
    async fn get_last_stored_sequence(&self) -> Result<u64, String> {
        let storage = self.runtime_data.lock().unwrap();
        Ok(storage.last().map(|d| d.sequence_number).unwrap_or(0))
    }
    
    async fn health_check(&self) -> Result<(), String> {
        if *self.should_fail.lock().unwrap() {
            Err("Mock health check failed".to_string())
        } else {
            Ok(())
        }
    }
}
```

## 7. 使用示例

### 7.1 生产环境（使用 SQLite）

```rust
use std::sync::Arc;
use crate::repositories::sqlite_storage_repository::SqliteStorageRepository;
use crate::pipeline::storage_pipeline::{StoragePipeline, StoragePipelineConfig};

async fn create_production_pipeline() -> Result<StoragePipeline, String> {
    // 创建 SQLite 仓库
    let repository = Arc::new(
        SqliteStorageRepository::new("data/crane_monitor.db").await?
    );
    
    // 创建存储管道（依赖注入）
    let config = StoragePipelineConfig::default();
    let buffer = create_shared_buffer();
    
    let pipeline = StoragePipeline::new(
        config,
        repository as Arc<dyn StorageRepository>,  // 向上转型为 trait object
        buffer,
    )?;
    
    Ok(pipeline)
}
```

### 7.2 测试环境（使用 Mock）

```rust
use std::sync::Arc;
use crate::repositories::mock_storage_repository::MockStorageRepository;
use crate::pipeline::storage_pipeline::{StoragePipeline, StoragePipelineConfig};

#[tokio::test]
async fn test_storage_pipeline() {
    // 创建 Mock 仓库
    let repository = Arc::new(MockStorageRepository::new());
    
    // 创建存储管道
    let config = StoragePipelineConfig::default();
    let buffer = create_test_buffer();
    
    let mut pipeline = StoragePipeline::new(
        config,
        repository.clone() as Arc<dyn StorageRepository>,
        buffer,
    ).unwrap();
    
    // 启动管道
    pipeline.start();
    
    // 等待数据处理
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 验证数据已存储
    assert!(repository.get_runtime_data_count() > 0);
    
    // 停止管道
    pipeline.stop();
}
```

### 7.3 切换数据库实现

```rust
// 可以轻松切换到其他数据库实现
// 例如：PostgreSQL、MySQL、内存数据库等

// PostgreSQL 实现
pub struct PostgresStorageRepository {
    // ...
}

#[async_trait]
impl StorageRepository for PostgresStorageRepository {
    // 实现相同的接口
}

// 使用时只需更换注入的实现
let repository = Arc::new(PostgresStorageRepository::new("postgres://...").await?);
let pipeline = StoragePipeline::new(config, repository, buffer)?;
```

## 8. 依赖项

```toml
[dependencies]
# 异步 trait
async-trait = "0.1"

# SQLite
rusqlite = { version = "0.31", features = ["bundled"] }

# 异步运行时
tokio = { version = "1.42", features = ["full"] }
```

## 9. 文件结构

```
src/repositories/
├── mod.rs
├── storage_repository.rs           # trait 定义（新增）
├── sqlite_storage_repository.rs    # SQLite 实现（新增）
├── mock_storage_repository.rs      # Mock 实现（新增）
└── crane_data_repository.rs        # 现有的仓库

src/pipeline/
├── mod.rs
├── storage_pipeline.rs             # 存储管道（解耦版本）
├── storage_queue.rs
├── collection_pipeline.rs
└── shared_buffer.rs
```

## 10. 优势总结

### ✅ 易于测试
```rust
// 使用 Mock 进行单元测试，无需真实数据库
let mock_repo = Arc::new(MockStorageRepository::new());
let pipeline = StoragePipeline::new(config, mock_repo, buffer)?;
```

### ✅ 支持多种数据库
```rust
// SQLite
let repo = SqliteStorageRepository::new("data.db").await?;

// PostgreSQL
let repo = PostgresStorageRepository::new("postgres://...").await?;

// 内存数据库
let repo = InMemoryStorageRepository::new();
```

### ✅ 依赖倒置
```rust
// 管道依赖抽象接口，不依赖具体实现
pub struct StoragePipeline {
    repository: Arc<dyn StorageRepository>,  // 依赖抽象
}
```

### ✅ 业务逻辑与数据访问分离
```
StoragePipeline        → 业务逻辑（队列管理、调度）
StorageRepository      → 抽象接口
SqliteStorageRepository → 数据访问（SQL 操作）
```

---

**版本**: 3.0（解耦版本）
**日期**: 2026-03-20
**状态**: 设计完成 ✅
