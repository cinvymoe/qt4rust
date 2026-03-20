# 后台线程 2（存储管道）设计快速参考

## 核心设计

### 1. 双表结构

```
runtime_data 表（运行数据）
├── 存储方式：定时批量存储（1秒间隔）
├── 数据来源：共享缓冲区 → 存储队列
└── 特点：避免重复，使用 sequence_number 唯一标识

alarm_records 表（报警信息）
├── 存储方式：异步回调立即存储
├── 数据来源：采集管道检测到报警时直接调用
└── 特点：实时响应，不经过队列
```

### 2. 数据流

```
┌─────────────────────────────────────────────────────────────┐
│                    采集管道（100ms）                         │
│                                                              │
│  采集数据 → 处理 → 检测报警？                                │
│                      ├─ 是 → save_alarm_async()             │
│                      │         ↓                             │
│                      │    Tokio 异步任务                     │
│                      │         ↓                             │
│                      │    alarm_records 表（立即存储）       │
│                      │                                       │
│                      └─ 否 → 写入共享缓冲区                  │
└─────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────┐
│                    共享缓冲区（线程安全）                     │
│                Arc<RwLock<ProcessedDataBuffer>>             │
└─────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────┐
│                    存储管道（1秒）                           │
│                                                              │
│  1. 从共享缓冲区读取新数据                                   │
│     ↓                                                        │
│  2. 过滤已存储数据（sequence_number > last_stored）         │
│     ↓                                                        │
│  3. 添加到存储队列                                           │
│     ↓                                                        │
│  4. 批量取出数据（peek_batch）                               │
│     ↓                                                        │
│  5. 异步批量存储到 runtime_data 表（使用事务）               │
│     ↓                                                        │
│  6. 存储成功后删除队列数据（remove_stored）                  │
│     ↓                                                        │
│  7. 更新 last_stored_sequence                               │
└─────────────────────────────────────────────────────────────┘
```

### 3. 避免重复存储的机制

```rust
// 存储队列追踪已存储的序列号
pub struct StorageQueue {
    queue: Arc<Mutex<VecDeque<ProcessedData>>>,
    last_stored_sequence: Arc<Mutex<u64>>,  // 关键：追踪已存储位置
}

// 添加数据时检查
pub fn push(&self, data: ProcessedData) -> Result<(), String> {
    if let Ok(last_seq) = self.last_stored_sequence.lock() {
        if data.sequence_number <= *last_seq {
            return Ok(()); // 已存储，跳过
        }
    }
    // 添加到队列...
}

// 存储成功后更新
pub fn remove_stored(&self, count: usize, max_sequence: u64) -> Result<(), String> {
    // 删除队列数据
    for _ in 0..count {
        queue.pop_front();
    }
    
    // 更新最后存储的序列号
    *last_stored_sequence = max_sequence;
}
```

### 4. 数据库表结构

```sql
-- 运行数据表
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

-- 报警信息表
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

### 5. 关键代码片段

#### 5.1 采集管道检测报警并回调

```rust
// 在采集管道中
let processed = ProcessedData::from_sensor_data(sensor_data, seq);

// 检测报警状态，立即异步存储
if processed.is_danger {
    eprintln!("[ALARM] Danger detected! Moment: {:.1}%", processed.moment_percentage);
    storage_pipeline.save_alarm_async(processed.clone());
}

// 写入共享缓冲区（正常流程）
if let Ok(mut buf) = buffer.write() {
    buf.push(processed);
}
```

#### 5.2 存储管道批量存储

```rust
// 存储管道主循环
while running.load(Ordering::Relaxed) {
    // 1. 从共享缓冲区读取新数据
    if let Ok(buf) = buffer.read() {
        let last_seq = storage_queue.last_stored_sequence();
        let new_data = buf.get_history(config.batch_size)
            .into_iter()
            .filter(|d| d.sequence_number > last_seq)  // 过滤已存储
            .collect::<Vec<_>>();
        
        // 2. 添加到队列
        for data in new_data {
            storage_queue.push(data)?;
        }
    }
    
    // 3. 批量取出数据
    let data_to_store = storage_queue.peek_batch(config.batch_size);
    
    if !data_to_store.is_empty() {
        let max_sequence = data_to_store.iter()
            .map(|d| d.sequence_number)
            .max()
            .unwrap_or(0);
        
        // 4. 异步存储
        tokio_runtime.spawn(async move {
            match db_source.save_runtime_data_batch(&data_clone).await {
                Ok(_) => {
                    // 5. 存储成功，删除队列数据
                    storage_queue.remove_stored(count, max_sequence)?;
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to save: {}", e);
                }
            }
        });
    }
    
    // 6. 控制频率
    thread::sleep(config.interval);
}
```

#### 5.3 异步报警回调

```rust
pub fn save_alarm_async(&self, data: ProcessedData) {
    let db_source = Arc::clone(&self.db_source);
    
    self.tokio_runtime.spawn(async move {
        if let Err(e) = db_source.save_alarm_record(&data).await {
            eprintln!("[ERROR] Failed to save alarm: {}", e);
        }
    });
}
```

#### 5.4 事务批量插入

```rust
pub fn save_runtime_data_batch(&self, data: &[ProcessedData]) -> Result<(), String> {
    let conn = self.connection.lock()?;
    
    // 开始事务
    conn.execute("BEGIN TRANSACTION", [])?;
    
    for item in data {
        conn.execute(
            "INSERT OR IGNORE INTO runtime_data (...) VALUES (...)",
            params![...],
        )?;
    }
    
    // 提交事务
    conn.execute("COMMIT", [])?;
    Ok(())
}
```

## 性能对比

| 指标 | 改进前 | 改进后 | 提升 |
|------|--------|--------|------|
| 重复存储 | 可能重复 | 完全避免 | ✅ 100% |
| 报警响应 | 延迟 1 秒 | 立即存储 | ✅ 实时 |
| 存储效率 | 单条插入 | 事务批量 | ✅ 10-100x |
| 内存占用 | 无限增长 | 队列限制 | ✅ 可控 |
| 数据一致性 | 无保证 | 事务保护 | ✅ 强一致 |

## 配置参数

```yaml
storage:
  interval_ms: 1000           # 存储间隔
  batch_size: 10              # 批量大小
  max_queue_size: 1000        # 队列容量
  max_retries: 3              # 重试次数
  retry_delay_ms: 100         # 重试延迟
  db_path: "data/crane_monitor.db"
```

## 依赖项

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
tokio = { version = "1.42", features = ["full"] }
```

## 文件清单

```
src/pipeline/
├── storage_queue.rs            # 存储队列（新增）
├── storage_pipeline.rs         # 存储管道（改进）
└── collection_pipeline.rs      # 采集管道（添加报警回调）

src/data_sources/
└── sqlite_data_source.rs       # SQLite 数据源（新增）
```

---

**版本**: 2.0  
**日期**: 2026-03-20  
**状态**: 设计完成 ✅
