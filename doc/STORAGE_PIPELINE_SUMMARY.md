# 后台线程 2（存储管道）设计总结

## 📋 设计概览

后台线程 2 负责数据持久化，采用解耦设计，支持双表存储和异步回调。

## 🎯 核心需求

1. ✅ 两个 SQL 表：运行数据表 + 报警信息表
2. ✅ 报警信息：异步回调方式立即存储
3. ✅ 运行数据：通过管道定时批量存储（1秒间隔）
4. ✅ 避免重复存储：使用序列号追踪
5. ✅ 数据库操作解耦：使用 trait 抽象

## 🏗️ 架构设计

### 层次结构

```
StoragePipeline（管道逻辑层）
    ↓ 依赖抽象
StorageRepository trait（存储接口层）
    ↓ 具体实现
SqliteStorageRepository（SQLite 实现）
MockStorageRepository（测试实现）
```

### 数据流

```
采集管道（100ms）
    ↓
检测数据
    ├─ 正常数据 → 共享缓冲区 → 存储队列 → 批量存储（1s）
    │                                    ↓
    │                              repository.save_runtime_data_batch()
    │                                    ↓
    │                              runtime_data 表
    │
    └─ 报警数据 → 异步回调 → repository.save_alarm_record()
                                    ↓
                              alarm_records 表
```

## 💾 数据库设计

### runtime_data 表（运行数据）

```sql
CREATE TABLE runtime_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL UNIQUE,  -- 防止重复
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
);
```

### alarm_records 表（报警信息）

```sql
CREATE TABLE alarm_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    alarm_type TEXT NOT NULL,  -- 'warning' 或 'danger'
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
);
```

## 🔧 核心组件

### 1. StorageRepository trait（抽象接口）

```rust
#[async_trait]
pub trait StorageRepository: Send + Sync {
    async fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<usize, String>;
    async fn save_alarm_record(&self, data: &ProcessedData) -> Result<i64, String>;
    async fn query_recent_runtime_data(&self, limit: usize) -> Result<Vec<ProcessedData>, String>;
    async fn query_unacknowledged_alarms(&self) -> Result<Vec<AlarmRecord>, String>;
    async fn acknowledge_alarm(&self, alarm_id: i64) -> Result<(), String>;
    async fn get_last_stored_sequence(&self) -> Result<u64, String>;
    async fn health_check(&self) -> Result<(), String>;
}
```

### 2. StorageQueue（存储队列）

```rust
pub struct StorageQueue {
    queue: Arc<Mutex<VecDeque<ProcessedData>>>,
    last_stored_sequence: Arc<Mutex<u64>>,  // 追踪已存储位置
}

// 关键方法
- push(): 添加数据（自动过滤已存储）
- peek_batch(): 批量取出数据（不删除）
- remove_stored(): 删除已存储数据，更新序列号
```

### 3. StoragePipeline（存储管道）

```rust
pub struct StoragePipeline {
    repository: Arc<dyn StorageRepository>,  // 依赖抽象接口
    storage_queue: Arc<StorageQueue>,
    // ...
}

// 关键方法
- start(): 启动管道（定时批量存储）
- stop(): 停止管道
- save_alarm_async(): 异步回调存储报警
```

## ✨ 关键特性

### 1. 避免重复存储

```rust
// 存储队列自动过滤
pub fn push(&self, data: ProcessedData) -> Result<(), String> {
    if data.sequence_number <= *last_stored_sequence {
        return Ok(()); // 已存储，跳过
    }
    queue.push_back(data);
}
```

### 2. 异步报警回调

```rust
// 采集管道检测到报警
if processed.is_danger {
    storage_pipeline.save_alarm_async(processed.clone());
}

// 存储管道立即存储
pub fn save_alarm_async(&self, data: ProcessedData) {
    tokio_runtime.spawn(async move {
        repository.save_alarm_record(&data).await?;
    });
}
```

### 3. 事务批处理

```rust
// 使用 SQLite 事务
conn.execute("BEGIN TRANSACTION", [])?;
for item in data {
    conn.execute("INSERT OR IGNORE INTO runtime_data ...", params![...])?;
}
conn.execute("COMMIT", [])?;
```

### 4. 依赖注入

```rust
// 生产环境
let repository = Arc::new(SqliteStorageRepository::new("data.db").await?);
let pipeline = StoragePipeline::new(config, repository, buffer)?;

// 测试环境
let repository = Arc::new(MockStorageRepository::new());
let pipeline = StoragePipeline::new(config, repository, buffer)?;
```

## 📊 性能优势

| 指标 | 改进前 | 改进后 | 提升 |
|------|--------|--------|------|
| 重复存储 | 可能重复 | 完全避免 | ✅ 100% |
| 报警响应 | 延迟 1 秒 | 立即存储 | ✅ 实时 |
| 存储效率 | 单条插入 | 事务批量 | ✅ 10-100x |
| 可测试性 | 依赖数据库 | Mock 测试 | ✅ 独立测试 |
| 扩展性 | 耦合 SQLite | 支持多种数据库 | ✅ 灵活切换 |

## 🧪 测试策略

### 单元测试（使用 Mock）

```rust
#[tokio::test]
async fn test_storage_pipeline() {
    let mock_repo = Arc::new(MockStorageRepository::new());
    let pipeline = StoragePipeline::new(config, mock_repo.clone(), buffer)?;
    
    pipeline.start();
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    assert!(mock_repo.get_runtime_data_count() > 0);
    pipeline.stop();
}
```

### 集成测试（使用真实数据库）

```rust
#[tokio::test]
async fn test_sqlite_integration() {
    let repo = Arc::new(SqliteStorageRepository::new(":memory:").await?);
    let pipeline = StoragePipeline::new(config, repo, buffer)?;
    
    // 测试完整流程
}
```

## 📦 依赖项

```toml
[dependencies]
async-trait = "0.1"
rusqlite = { version = "0.31", features = ["bundled"] }
tokio = { version = "1.42", features = ["full"] }
```

## 📁 文件结构

```
src/
├── repositories/
│   ├── storage_repository.rs           # trait 定义
│   ├── sqlite_storage_repository.rs    # SQLite 实现
│   └── mock_storage_repository.rs      # Mock 实现
├── pipeline/
│   ├── storage_pipeline.rs             # 存储管道
│   ├── storage_queue.rs                # 存储队列
│   └── collection_pipeline.rs          # 采集管道（添加报警回调）
└── models/
    ├── processed_data.rs
    └── alarm_record.rs
```

## 🚀 使用示例

### 初始化

```rust
// 创建 SQLite 仓库
let repository = Arc::new(
    SqliteStorageRepository::new("data/crane_monitor.db").await?
);

// 创建存储管道
let pipeline = StoragePipeline::new(
    config,
    repository as Arc<dyn StorageRepository>,
    buffer,
)?;

// 启动管道
pipeline.start();
```

### 查询数据

```rust
// 查询最近的运行数据
let data = repository.query_recent_runtime_data(100).await?;

// 查询未确认的报警
let alarms = repository.query_unacknowledged_alarms().await?;

// 确认报警
repository.acknowledge_alarm(alarm_id).await?;
```

## 📚 相关文档

1. **doc/THREE_BACKEND_PIPELINE_ARCHITECTURE.md** - 完整架构文档
2. **doc/STORAGE_DECOUPLING_DESIGN.md** - 解耦设计详解
3. **doc/STORAGE_PIPELINE_DESIGN.md** - 快速参考指南

## ✅ 设计检查清单

- [x] 双表设计（runtime_data + alarm_records）
- [x] 避免重复存储（sequence_number 追踪）
- [x] 异步报警回调（立即存储）
- [x] 事务批处理（提升性能）
- [x] 数据库操作解耦（StorageRepository trait）
- [x] 依赖注入（易于测试）
- [x] Mock 实现（单元测试）
- [x] 完整的代码示例
- [ ] 单元测试实现
- [ ] 集成测试实现

---

**版本**: 3.0（解耦版本）  
**日期**: 2026-03-20  
**状态**: 设计完成 ✅  
**下一步**: 开始实现代码
