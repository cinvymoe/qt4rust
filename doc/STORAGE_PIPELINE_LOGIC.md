# 存储管道采集逻辑详解

## 概述

存储管道（Storage Pipeline）是三管道架构中的第二个管道，负责将采集管道生成的数据持久化到 SQLite 数据库。

## 核心配置

```rust
StoragePipelineConfig {
    interval: 5000ms,        // 存储间隔：每 5 秒执行一次
    batch_size: 10,          // 批量大小：每次最多处理 10 条数据
    max_retries: 3,          // 失败重试次数
    retry_delay: 100ms,      // 重试延迟
    max_queue_size: 1000,    // 队列最大容量
}
```

## 数据流程图

```
┌─────────────────────────────────────────────────────────────────┐
│                    存储管道主循环 (每 5 秒)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 步骤 1: 从共享缓冲区读取新数据                                   │
├─────────────────────────────────────────────────────────────────┤
│ 1. 获取 last_stored_sequence (例如: 16912)                      │
│ 2. 从 SharedBuffer 读取最新 10 条数据                           │
│ 3. 过滤出 sequence_number > last_stored_sequence 的数据         │
│ 4. 将新数据推入 StorageQueue                                    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 步骤 2: 从队列取出数据批量存储                                   │
├─────────────────────────────────────────────────────────────────┤
│ 1. 从 StorageQueue 取出最多 10 条数据 (peek_batch)              │
│ 2. 找出最大序列号 (max_sequence)                                │
│ 3. 异步调用 repository.save_runtime_data_batch()                │
│ 4. 存储成功后，从队列删除已保存的数据                            │
│ 5. 更新 last_stored_sequence = max_sequence                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│ 步骤 3: 控制存储频率                                             │
├─────────────────────────────────────────────────────────────────┤
│ 1. 计算本次迭代耗时                                             │
│ 2. 如果耗时 < 5 秒，休眠剩余时间                                │
│ 3. 回到步骤 1                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 详细步骤说明

### 步骤 1: 从共享缓冲区读取新数据

```rust
// 1.1 获取最后存储的序列号
let last_seq = storage_queue.last_stored_sequence();  // 例如: 16912

// 1.2 从 SharedBuffer 读取最新数据
let history = buf.get_history(config.batch_size);  // 读取最新 10 条

// 1.3 过滤出新数据
let new_data = history
    .into_iter()
    .filter(|d| d.sequence_number > last_seq)  // 只要 > 16912 的数据
    .collect::<Vec<_>>();

// 1.4 推入存储队列
for data in new_data {
    storage_queue.push(data);
}
```

**关键点**：
- SharedBuffer 是一个环形缓冲区，只保留最新的 N 条数据（默认 100 条）
- 每次读取 `batch_size` 条数据（默认 10 条）
- 通过 `sequence_number` 过滤，避免重复存储

### 步骤 2: 批量存储到数据库

```rust
// 2.1 从队列取出数据（不删除）
let data_to_store = storage_queue.peek_batch(config.batch_size);  // 最多 10 条

// 2.2 找出最大序列号
let max_sequence = data_to_store.iter()
    .map(|d| d.sequence_number)
    .max()
    .unwrap_or(0);  // 例如: 16962

// 2.3 异步存储到数据库
tokio_runtime.spawn(async move {
    match repository.save_runtime_data_batch(&data_clone).await {
        Ok(saved_count) => {
            // 2.4 存储成功，从队列删除
            storage_queue.remove_stored(count, max_sequence);
            // 这会更新 last_stored_sequence = 16962
        }
        Err(e) => {
            // 存储失败，数据保留在队列中，下次重试
        }
    }
});
```

**关键点**：
- 使用 `peek_batch` 而不是直接删除，确保存储成功后才删除
- 异步存储，不阻塞主循环
- 存储失败时数据保留在队列中，下次迭代会重试

### 步骤 3: 频率控制

```rust
// 3.1 计算本次迭代耗时
let elapsed = start_time.elapsed();  // 例如: 0.025s

// 3.2 计算需要休眠的时间
let sleep_time = if elapsed < config.interval {
    config.interval - elapsed  // 5.0s - 0.025s = 4.975s
} else {
    Duration::from_millis(0)
};

// 3.3 休眠
thread::sleep(sleep_time);
```

**关键点**：
- 确保每次迭代间隔至少 5 秒
- 如果处理时间超过 5 秒，立即开始下一次迭代

## 实际运行示例

### 时间线示例

```
T=0s    [采集管道] 生成数据 seq=16913-16922 → SharedBuffer
T=0.1s  [采集管道] 生成数据 seq=16923-16932 → SharedBuffer
T=0.2s  [采集管道] 生成数据 seq=16933-16942 → SharedBuffer
...
T=5.0s  [存储管道] 第 1 次迭代
        - 从 SharedBuffer 读取最新 10 条: seq=16933-16942
        - 过滤: 16933-16942 > 16912 (last_seq) ✓ 全部保留
        - 推入 StorageQueue: 10 条
        - 从 StorageQueue 取出 10 条
        - 保存到数据库: 10 条
        - 更新 last_seq = 16942
        - 休眠 4.975s

T=10.0s [存储管道] 第 2 次迭代
        - 从 SharedBuffer 读取最新 10 条: seq=16983-16992
        - 过滤: 16983-16992 > 16942 (last_seq) ✓ 全部保留
        - 推入 StorageQueue: 10 条
        - 从 StorageQueue 取出 10 条
        - 保存到数据库: 10 条
        - 更新 last_seq = 16992
        - 休眠 4.975s
```

### 日志示例

```
[DEBUG] Storage: buffer_size=10, seq_range=[16953, 16962], last_stored=16912
[DEBUG] Storage: last_seq=16912, new_data_count=10
[DEBUG] Storage: queue_len=10, data_to_store=10
[DEBUG] Attempting to save 10 records
[INFO]  Saved 10 records
[DEBUG] Removed 10 records from queue, last_seq=16962
[INFO]  [STORAGE] Iteration #0 completed in 0.025s, sleeping for 4.975s
```

## 数据一致性保证

### 1. 序列号连续性

- 采集管道使用 `AtomicU64` 生成递增序列号
- 程序重启时，从数据库读取最大序列号并继续
- 存储管道通过 `sequence_number` 过滤，确保不重复存储

### 2. 数据不丢失

- SharedBuffer 保留最新 100 条数据（可配置）
- StorageQueue 最多缓存 1000 条待存储数据
- 存储失败时数据保留在队列中，下次重试

### 3. 并发安全

- SharedBuffer 使用 `Arc<RwLock<>>` 保护
- StorageQueue 使用 `Arc<Mutex<>>` 保护
- 异步存储不阻塞主循环

## 性能特点

### 优点

1. **批量存储**：每次存储 10 条数据，减少数据库 I/O
2. **异步处理**：存储操作不阻塞主循环
3. **频率可控**：5 秒间隔，避免频繁写入数据库
4. **内存占用低**：只缓存必要的数据

### 缺点与改进

1. **数据延迟**：最多 5 秒延迟（可接受）
2. **丢失风险**：程序崩溃时，队列中的数据可能丢失
   - 改进：可以添加 WAL（Write-Ahead Log）机制

## 配置调优建议

### 高频存储（实时性优先）

```rust
StoragePipelineConfig {
    interval: Duration::from_secs(1),  // 1 秒存储一次
    batch_size: 5,                     // 每次 5 条
    ...
}
```

### 低频存储（性能优先）

```rust
StoragePipelineConfig {
    interval: Duration::from_secs(10), // 10 秒存储一次
    batch_size: 50,                    // 每次 50 条
    ...
}
```

### 当前配置（平衡）

```rust
StoragePipelineConfig {
    interval: Duration::from_secs(5),  // 5 秒存储一次
    batch_size: 10,                    // 每次 10 条
    ...
}
```

## 与采集管道的协作

```
采集管道 (10Hz)              存储管道 (0.2Hz)
    │                            │
    ├─ 100ms ─> 数据 seq=1       │
    ├─ 100ms ─> 数据 seq=2       │
    ├─ 100ms ─> 数据 seq=3       │
    │           ...              │
    ├─ 100ms ─> 数据 seq=50      │
    │                            ├─ 5s ─> 保存 seq=1-10
    ├─ 100ms ─> 数据 seq=51      │
    │           ...              │
    ├─ 100ms ─> 数据 seq=100     │
    │                            ├─ 5s ─> 保存 seq=11-20
    │           ...              │
```

**关键点**：
- 采集管道：100ms 生成 1 条数据（10Hz）
- 存储管道：5000ms 保存 10 条数据（0.2Hz）
- 采集速度 = 10 条/秒，存储速度 = 2 条/秒
- SharedBuffer 缓冲区平衡两者速度差异

## 总结

存储管道的采集逻辑核心是：

1. **定时轮询**：每 5 秒从 SharedBuffer 读取新数据
2. **增量存储**：只存储 `sequence_number > last_stored_sequence` 的数据
3. **批量处理**：每次最多处理 10 条数据
4. **异步存储**：不阻塞主循环
5. **失败重试**：存储失败时数据保留在队列中

这种设计在实时性、性能和可靠性之间取得了良好的平衡。
