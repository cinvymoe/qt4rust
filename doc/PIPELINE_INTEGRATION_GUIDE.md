# 管道系统集成指南

## 概述

本文档说明如何使用完整的三后台管道架构系统。

## 架构组件

### 1. 后台线程 1：采集管道（Collection Pipeline）
- **频率**: 10Hz (100ms 间隔)
- **功能**: 从传感器采集数据，处理后写入共享缓冲区
- **报警检测**: 检测危险状态，触发报警回调

### 2. 后台线程 2：存储管道（Storage Pipeline）
- **频率**: 1Hz (1s 间隔)
- **功能**: 
  - 批量存储运行数据到 `runtime_data` 表
  - 异步回调立即存储报警到 `alarm_records` 表
- **避免重复**: 使用 `sequence_number` 追踪已存储数据

### 3. 主线程：显示管道（Display Pipeline）
- **状态**: 待实现
- **功能**: 从共享缓冲区读取数据，更新 UI

## 使用方法

### 基本用法（不带存储）

```rust
use std::sync::Arc;
use crane_hmi::repositories::CraneDataRepository;
use crane_hmi::pipeline::pipeline_manager::PipelineManager;

fn main() {
    // 创建数据仓库
    let repository = Arc::new(CraneDataRepository::default());
    
    // 创建管道管理器（仅采集）
    let mut manager = PipelineManager::new(repository);
    
    // 启动采集管道
    manager.start_collection_pipeline();
    
    // ... 运行 ...
    
    // 停止
    manager.stop_collection_pipeline();
}
```

### 完整用法（带存储）

```rust
use std::sync::Arc;
use crane_hmi::repositories::CraneDataRepository;
use crane_hmi::pipeline::pipeline_manager::PipelineManager;

#[tokio::main]
async fn main() -> Result<(), String> {
    // 创建数据仓库
    let repository = Arc::new(CraneDataRepository::default());
    
    // 创建管道管理器（带存储支持）
    let mut manager = PipelineManager::new_with_storage(
        repository,
        "crane_data.db",  // 数据库路径
    ).await?;
    
    // 启动所有管道（采集 + 存储）
    manager.start_all();
    
    // ... 运行 ...
    
    // 停止所有管道
    manager.stop_all();
    
    Ok(())
}
```

### 监控管道状态

```rust
// 检查管道运行状态
if manager.is_collection_running() {
    println!("采集管道运行中");
}

if manager.is_storage_running() {
    println!("存储管道运行中");
}

// 获取存储队列长度
if let Some(queue_len) = manager.get_storage_queue_len() {
    println!("存储队列: {} 条数据", queue_len);
}

// 获取最后存储的序列号
if let Some(last_seq) = manager.get_last_stored_sequence() {
    println!("最后存储序列号: {}", last_seq);
}

// 获取共享缓冲区统计
let buffer = manager.get_shared_buffer();
let stats = buffer.read().unwrap().get_stats();
println!("采集统计: 总数={}, 成功={}, 失败={}",
         stats.total_collections,
         stats.success_count,
         stats.error_count);
```

## 数据流

```
传感器
  ↓
采集管道 (10Hz)
  ↓
共享缓冲区 (1000条)
  ↓
存储管道 (1Hz)
  ↓
SQLite 数据库
  ├─ runtime_data (运行数据)
  └─ alarm_records (报警记录)
```

## 报警处理流程

```
采集管道检测到危险
  ↓
触发报警回调
  ↓
存储管道异步保存
  ↓
立即写入 alarm_records 表
```

## 避免重复存储机制

1. 每条数据有唯一的 `sequence_number`
2. 存储队列记录 `last_stored_sequence`
3. 从共享缓冲区读取时，只取 `sequence_number > last_stored_sequence` 的数据
4. 存储成功后，更新 `last_stored_sequence` 并从队列删除已存储数据

## 配置选项

### 采集管道配置

```rust
use crane_hmi::pipeline::collection_pipeline::CollectionPipelineConfig;
use std::time::Duration;

let config = CollectionPipelineConfig {
    interval: Duration::from_millis(100),  // 采集间隔
    max_retries: 3,                        // 最大重试次数
    disconnect_threshold: 10,              // 断线阈值
};
```

### 存储管道配置

```rust
use crane_hmi::pipeline::storage_pipeline::StoragePipelineConfig;
use std::time::Duration;

let config = StoragePipelineConfig {
    interval: Duration::from_secs(1),      // 存储间隔
    batch_size: 10,                        // 批量大小
    max_retries: 3,                        // 最大重试次数
    retry_delay: Duration::from_millis(100), // 重试延迟
    max_queue_size: 1000,                  // 队列最大容量
};
```

## 数据库表结构

### runtime_data 表

```sql
CREATE TABLE runtime_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL UNIQUE,
    timestamp INTEGER NOT NULL,
    current_load REAL NOT NULL,
    rated_load REAL NOT NULL,
    working_radius REAL NOT NULL,
    boom_angle REAL NOT NULL,
    boom_length REAL NOT NULL,
    moment_percentage REAL NOT NULL,
    is_danger INTEGER NOT NULL
);
```

### alarm_records 表

```sql
CREATE TABLE alarm_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sequence_number INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    current_load REAL NOT NULL,
    rated_load REAL NOT NULL,
    working_radius REAL NOT NULL,
    boom_angle REAL NOT NULL,
    moment_percentage REAL NOT NULL,
    acknowledged INTEGER NOT NULL DEFAULT 0,
    acknowledged_at INTEGER
);
```

## 运行示例

```bash
# 运行完整管道演示
cargo run --example full_pipeline_demo

# 查看数据库内容
sqlite3 crane_data.db "SELECT * FROM runtime_data LIMIT 10;"
sqlite3 crane_data.db "SELECT * FROM alarm_records;"
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行管道管理器测试
cargo test --test pipeline_manager

# 运行存储管道测试
cargo test --package crane-hmi --lib pipeline::storage_pipeline
```

## 故障排查

### 问题：存储队列持续增长

**原因**: 存储速度慢于采集速度

**解决方案**:
- 增加 `batch_size`
- 减少 `interval`
- 检查数据库性能

### 问题：报警未保存

**原因**: 报警回调未正确连接

**解决方案**:
- 确保先启动存储管道，再启动采集管道
- 或使用 `start_all()` 方法

### 问题：数据重复存储

**原因**: `sequence_number` 追踪失败

**解决方案**:
- 检查 `last_stored_sequence` 是否正确更新
- 查看日志确认存储成功

## 性能指标

- **采集频率**: 10Hz (100ms)
- **存储频率**: 1Hz (1s)
- **批量大小**: 10 条/批次
- **缓冲区容量**: 1000 条
- **队列容量**: 1000 条

## 下一步

- [ ] 实现显示管道（主线程）
- [ ] 添加性能监控
- [ ] 实现数据导出功能
- [ ] 添加配置文件支持
