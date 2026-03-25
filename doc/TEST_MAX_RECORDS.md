# 数据库最大记录条数功能测试指南

## 功能概述

数据库最大记录条数功能用于自动清理旧数据，防止数据库无限增长。

### 配置参数

在 `config/pipeline_config.toml` 中配置：

```toml
[storage]
# 数据库最大记录条数（0 表示不限制）
max_records = 100

# 清理阈值（0 表示使用默认值 max_records * 1.1）
purge_threshold = 0
```

### 工作原理

1. 每次存储数据后，检查 `max_records` 是否大于 0
2. 如果大于 0，调用 `purge_old_records()` 检查是否需要清理
3. 当记录数超过 `purge_threshold` 时，删除最早的记录
4. 删除到记录数等于 `max_records` 为止

### 默认阈值计算

如果 `purge_threshold = 0`，使用默认值：

```
threshold = min(max_records * 1.1, max_records + 1000)
```

例如：
- `max_records = 100` → `threshold = 110`
- `max_records = 10000` → `threshold = 11000`

## 测试方法

### 方法 1: 单元测试（快速验证）

测试基本的清理逻辑：

```bash
# 运行单元测试
cargo run --example test_max_records
```

**测试场景：**

1. **场景 1**: `max_records=10, purge_threshold=0`
   - 插入 15 条记录
   - 验证最终记录数 ≤ 10

2. **场景 2**: `max_records=20, purge_threshold=25`
   - 插入 30 条记录
   - 验证清理在记录数超过 25 时触发

3. **场景 3**: `max_records=0`（不限制）
   - 插入 50 条记录
   - 验证不删除任何记录

**预期输出：**

```
========================================
  数据库最大记录条数功能测试
========================================

【测试 1】max_records=10, purge_threshold=0 (默认 11)
  配置: max_records=10, purge_threshold=0
  插入 15 条记录...
    [第 10 条] 清理了 0 条旧记录
    [第 15 条] 清理了 5 条旧记录
  最终清理: 删除了 0 条旧记录
  ✓ 当前记录数: 10
  ✓ 测试通过: 记录数 10 <= max_records 10
  最早记录: sequence=6
  最晚记录: sequence=15

【测试 2】max_records=20, purge_threshold=25
  配置: max_records=20, purge_threshold=25
  插入 30 条记录...
    [第 25 条] 清理了 5 条旧记录
    [第 30 条] 清理了 5 条旧记录
  ✓ 当前记录数: 20
  ✓ 测试通过: 记录数 20 <= max_records 20

【测试 3】max_records=0 (不限制)
  配置: max_records=0 (不限制)
  插入 50 条记录...
  清理结果: 删除了 0 条记录
  ✓ 当前记录数: 50
  ✓ 测试通过: max_records=0 时不清理数据

========================================
  所有测试完成！
========================================
```

### 方法 2: 管道集成测试（真实场景）

测试在实际管道运行中的表现：

```bash
# 运行管道集成测试
cargo run --example test_max_records_pipeline
```

**测试配置：**

- `max_records = 50`
- `purge_threshold = 55`
- 采集间隔: 100ms
- 存储间隔: 1000ms
- 批量大小: 5
- 测试时长: 30秒

**预期输出：**

```
========================================
  管道集成测试 - 最大记录条数
========================================

测试配置:
  - max_records: 50
  - purge_threshold: 55
  - 采集间隔: 100ms
  - 存储间隔: 1000ms
  - 批量大小: 5
  - 测试时长: 30秒

启动管道...

[5s] 数据库记录数: 25, 队列长度: 0, 最后序列号: 25
[10s] 数据库记录数: 50, 队列长度: 0, 最后序列号: 50
[15s] 数据库记录数: 50, 队列长度: 0, 最后序列号: 75
[20s] 数据库记录数: 50, 队列长度: 0, 最后序列号: 100
[25s] 数据库记录数: 50, 队列长度: 0, 最后序列号: 125
[30s] 数据库记录数: 50, 队列长度: 0, 最后序列号: 150

停止管道...

========================================
  测试结果
========================================
最终记录数: 50
max_records: 50

✓ 测试通过: 记录数 50 <= max_records 50

最早记录:
  - sequence: 101
  - load: 111.0t

最晚记录:
  - sequence: 150
  - load: 160.0t

提示: 查看日志中的 'Purged' 信息了解清理详情
数据库文件: test_pipeline_max_records.db

========================================
```

**日志中的清理信息：**

```
INFO  Saved 5 records
INFO  Purged 5 old records (max_records=50, threshold=55)
```

### 方法 3: 手动验证数据库

使用 SQL 脚本检查数据库：

```bash
# 给脚本添加执行权限
chmod +x examples/check_db_records.sh

# 检查默认数据库
./examples/check_db_records.sh

# 检查指定数据库
./examples/check_db_records.sh test_pipeline_max_records.db
```

**预期输出：**

```
========================================
  数据库记录统计
========================================
数据库文件: crane_data.db

【运行数据表】
总记录数: 100
序列号范围: 901 - 1000

最早的 5 条记录:
id  sequence_number  ad1_load  ad2_radius  timestamp
--  ---------------  --------  ----------  -------------------
1   901              911.0     8.0         2026-03-25 10:30:01
2   902              912.0     8.0         2026-03-25 10:30:02
3   903              913.0     8.0         2026-03-25 10:30:03
4   904              914.0     8.0         2026-03-25 10:30:04
5   905              915.0     8.0         2026-03-25 10:30:05

最晚的 5 条记录:
id   sequence_number  ad1_load  ad2_radius  timestamp
---  ---------------  --------  ----------  -------------------
100  1000             1010.0    8.0         2026-03-25 10:31:40
99   999              1009.0    8.0         2026-03-25 10:31:39
98   998              1008.0    8.0         2026-03-25 10:31:38
97   997              1007.0    8.0         2026-03-25 10:31:37
96   996              1006.0    8.0         2026-03-25 10:31:36

【报警记录表】
总记录数: 0

========================================
```

### 方法 4: 修改配置文件测试

1. **编辑配置文件**

```bash
vim config/pipeline_config.toml
```

修改以下参数：

```toml
[storage]
max_records = 50      # 改为 50
purge_threshold = 0   # 使用默认值 55
```

2. **运行主程序**

```bash
cargo run --target armv7-unknown-linux-gnueabihf
```

3. **观察日志**

查找包含 "Purged" 的日志行：

```bash
# 实时查看日志
cargo run 2>&1 | grep -i purge

# 或者查看日志文件
tail -f logs/app.log | grep -i purge
```

4. **定期检查数据库**

```bash
# 每 10 秒检查一次
watch -n 10 './examples/check_db_records.sh'
```

## 验证要点

### ✓ 功能正常的标志

1. **记录数稳定在 max_records 附近**
   - 不会无限增长
   - 不会频繁波动

2. **清理日志正常**
   ```
   INFO  Purged 5 old records (max_records=50, threshold=55)
   ```

3. **序列号连续增长**
   - 最早记录的序列号不断增加
   - 最晚记录的序列号持续递增

4. **性能正常**
   - 清理操作不阻塞数据采集
   - 存储延迟保持稳定

### ✗ 可能的问题

1. **记录数持续超过 max_records**
   - 检查 `purge_threshold` 是否设置过大
   - 检查清理逻辑是否被调用

2. **频繁清理（每次存储都清理）**
   - `purge_threshold` 设置过小
   - 建议设置为 `max_records * 1.1` 以上

3. **清理失败**
   - 查看错误日志
   - 检查数据库权限
   - 检查磁盘空间

## 性能测试

### 测试清理性能

```bash
# 测试大量数据的清理性能
cargo run --example test_max_records -- --max-records 10000 --insert-count 20000
```

**关注指标：**

- 清理操作耗时
- 数据库文件大小
- 内存使用情况

### 压力测试

```bash
# 高频采集 + 小 max_records
# 修改配置：
# - collection.interval_ms = 10
# - storage.interval_ms = 100
# - storage.max_records = 100

cargo run --target armv7-unknown-linux-gnueabihf
```

**观察：**

- CPU 使用率
- 存储队列长度
- 清理频率

## 常见问题

### Q1: 为什么记录数会超过 max_records？

A: 这是正常的。清理在记录数超过 `purge_threshold` 时才触发，然后删除到 `max_records`。这样可以避免频繁清理。

### Q2: purge_threshold 应该设置多少？

A: 建议设置为 `max_records * 1.1` 到 `max_records * 1.5` 之间。太小会频繁清理，太大会占用过多空间。

### Q3: 清理会影响性能吗？

A: 清理操作是异步的，不会阻塞数据采集。但如果 `max_records` 很小且采集频率很高，可能会频繁触发清理。

### Q4: 如何禁用自动清理？

A: 设置 `max_records = 0`。

### Q5: 清理会删除报警记录吗？

A: 不会。清理只针对 `runtime_data` 表，`alarm_records` 表不受影响。

## 配置建议

### 低频采集（1Hz）

```toml
[collection]
interval_ms = 1000

[storage]
interval_ms = 5000
batch_size = 5
max_records = 1000      # 约 16 分钟数据
purge_threshold = 0     # 默认 1100
```

### 中频采集（10Hz）

```toml
[collection]
interval_ms = 100

[storage]
interval_ms = 1000
batch_size = 10
max_records = 10000     # 约 16 分钟数据
purge_threshold = 0     # 默认 11000
```

### 高频采集（100Hz）

```toml
[collection]
interval_ms = 10

[storage]
interval_ms = 100
batch_size = 10
max_records = 100000    # 约 16 分钟数据
purge_threshold = 0     # 默认 110000
```

## 总结

数据库最大记录条数功能已实现并可以通过多种方式测试。建议按以下顺序进行：

1. 运行单元测试验证基本逻辑
2. 运行管道集成测试验证真实场景
3. 修改配置文件进行实际测试
4. 使用 SQL 脚本手动验证数据库

测试完成后，根据实际需求调整 `max_records` 和 `purge_threshold` 参数。
