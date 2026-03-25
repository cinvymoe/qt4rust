# 管道激活说明

## 修改内容

已在应用中同时开启管道1（采集管道）和管道2（存储管道）。

### 修改文件

- `src/viewmodel_manager.rs`

### 主要变更

1. **使用带存储支持的管道管理器**
   - 从 `PipelineManager::new()` 改为 `PipelineManager::new_with_storage()`
   - 自动初始化 SQLite 数据库（`crane_data.db`）

2. **启动所有管道**
   - 从 `manager.start_collection_pipeline()` 改为 `manager.start_all()`
   - 同时启动采集管道和存储管道

3. **错误处理**
   - 如果存储系统初始化失败，自动降级到仅采集模式
   - 确保应用在任何情况下都能正常运行

## 管道说明

### 管道1：采集管道（Collection Pipeline）
- **频率**: 10Hz (100ms 间隔)
- **功能**: 从传感器采集数据，写入共享缓冲区
- **线程**: 后台线程 1

### 管道2：存储管道（Storage Pipeline）
- **频率**: 1Hz (1秒间隔)
- **功能**: 
  - 批量存储运行数据到 SQLite（每次最多10条）
  - 异步存储报警记录
- **线程**: 后台线程 2
- **数据库**: `crane_data.db`

### 管道3：显示管道（Display Pipeline）
- **状态**: 待实现
- **功能**: 从共享缓冲区读取数据，更新 UI
- **线程**: Qt 主线程

## 数据流

```
传感器 → 采集管道(10Hz) → 共享缓冲区 → 存储管道(1Hz) → SQLite
                              ↓
                         显示管道 → UI
```

## 验证方法

### 1. 查看日志输出

启动应用后，应该看到以下日志：

```
[INFO] Starting three-pipeline data collection...
[INFO] Initializing storage system with database: crane_data.db
[INFO] Storage system initialized successfully
[INFO] Starting all pipelines...
[INFO] Starting storage pipeline (Backend Thread 2)...
[INFO] Storage pipeline started successfully
[INFO] - Interval: 1s
[INFO] - Batch size: 10
[INFO] Starting collection pipeline (Backend Thread 1)...
[INFO] Alarm callback connected to storage pipeline
[INFO] Collection pipeline started successfully
[INFO] - Interval: 100ms (10Hz)
[INFO] - Max retries: 3
[INFO] - Disconnect threshold: 10
[INFO] All pipelines started
[INFO] Three-pipeline data collection started
[INFO] Backend Thread 1 (Collection Pipeline) is now running at 10Hz
[INFO] Backend Thread 2 (Storage Pipeline) is now running at 1Hz
```

### 2. 检查数据库文件

运行应用后，应该在应用目录下生成 `crane_data.db` 文件：

```bash
ls -lh crane_data.db
```

### 3. 查询数据库内容

```bash
# 查看运行数据表
sqlite3 crane_data.db "SELECT COUNT(*) FROM runtime_data;"

# 查看最近10条运行数据
sqlite3 crane_data.db "SELECT * FROM runtime_data ORDER BY id DESC LIMIT 10;"

# 查看报警记录
sqlite3 crane_data.db "SELECT * FROM alarm_records;"
```

## 性能指标

- **采集频率**: 10Hz（每秒10次）
- **存储频率**: 1Hz（每秒1次，批量10条）
- **数据延迟**: 最大1秒（从采集到存储）
- **内存占用**: 共享缓冲区保留最近1000条数据

## 故障降级

如果存储系统初始化失败（例如数据库文件无法创建），应用会自动降级到仅采集模式：

```
[ERROR] Failed to initialize storage system: ...
[WARN] Falling back to collection-only mode
[INFO] Collection pipeline started successfully
```

在此模式下：
- 管道1（采集）正常运行
- 管道2（存储）不启动
- 数据仍然写入共享缓冲区
- UI 显示不受影响

## 下一步

- [ ] 实现管道3（显示管道）
- [ ] 添加管道状态监控 UI
- [ ] 添加数据库查询和导出功能
- [ ] 优化存储性能（批量大小、存储间隔）

---

**更新日期**: 2026-03-24
**状态**: ✅ 已完成
