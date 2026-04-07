# 两个存储管道说明

## 概述

系统中有**两个独立的存储管道**，分别存储不同类型的数据：

1. **ProcessedData 存储管道** - 存储处理后的数据
2. **SensorData 存储管道** - 存储原始传感器数据

## 对比表

| 特性 | ProcessedData 存储 | SensorData 存储 |
|------|-------------------|----------------|
| **配置节** | `[storage]` | `[sensor_storage]` |
| **batch_size** | 10 | 100 |
| **interval** | 1000ms (1秒) | 60秒 |
| **数据内容** | 处理后数据（含计算结果） | 原始传感器数据 |
| **日志标识** | `Saved X records` | `Saved X sensor data records` |
| **触发条件** | 攒够 10 条 OR 每 1 秒 | 攒够 100 条 OR 每 60 秒 |

## 1. ProcessedData 存储管道

### 配置

```toml
[storage]
interval_ms = 1000
batch_size = 10
```

### 存储内容

```rust
pub struct ProcessedData {
    pub current_load: f64,        // 当前载荷
    pub rated_load: f64,          // 额定载荷
    pub working_radius: f64,      // 工作半径
    pub boom_angle: f64,          // 吊臂角度
    pub boom_length: f64,         // 臂长
    pub moment_percentage: f64,   // 力矩百分比 ← 计算结果
    pub is_danger: bool,          // 是否危险 ← 计算结果
    pub sequence_number: u64,     // 序列号
    pub timestamp: SystemTime,    // 时间戳
}
```

### 日志示例

```
INFO Saved 10 records (seq <= 3321372)
```

### 触发时机

- **条件 1**: 攒够 10 条数据
- **条件 2**: 每 1 秒强制存储

**计算**：采集频率 100ms，攒够 10 条需要 1 秒
```
10 条 × 100ms = 1000ms = 1 秒
```

## 2. SensorData 存储管道

### 配置

```toml
[sensor_storage]
enabled = true
batch_size = 100
interval_secs = 60
max_records = 10000
```

### 存储内容

```rust
pub struct SensorData {
    pub ad1_load: f64,      // AD1 原始值（载荷）
    pub ad2_radius: f64,    // AD2 原始值（半径）
    pub ad3_angle: f64,     // AD3 原始值（角度）
}
```

### 日志示例

```
INFO Saved 100 sensor data records
```

### 触发时机

- **条件 1**: 攒够 100 条数据
- **条件 2**: 每 60 秒强制存储

**计算**：采集频率 100ms，攒够 100 条需要 10 秒
```
100 条 × 100ms = 10,000ms = 10 秒
```

## 数据流图

```
传感器采集 (100ms)
    ↓
SensorData (原始数据)
    ├─→ SensorData 存储管道
    │   ├─ 攒够 100 条 → 存储
    │   └─ 或每 60 秒 → 存储
    │
    └─→ 数据处理
        ↓
    ProcessedData (处理后数据)
        ↓
    ProcessedData 存储管道
        ├─ 攒够 10 条 → 存储
        └─ 或每 1 秒 → 存储
```

## 为什么需要两个存储管道？

### ProcessedData 存储

**用途**: 
- 实时监控
- 报警记录
- 力矩分析
- 危险状态追踪

**特点**:
- 高频存储（1 秒一次）
- 包含计算结果
- 用于实时分析

### SensorData 存储

**用途**:
- 原始数据备份
- 校准参数调整
- 数据回放
- 故障分析

**特点**:
- 低频存储（60 秒一次）
- 只存原始值
- 用于离线分析

## 配置调整建议

### 场景 1: 需要更频繁的原始数据存储

```toml
[sensor_storage]
batch_size = 50      # 减少 batch_size
interval_secs = 30   # 减少间隔
```

**效果**: 每 5 秒或攒够 50 条就存储

### 场景 2: 节省存储空间

```toml
[sensor_storage]
batch_size = 200     # 增加 batch_size
interval_secs = 120  # 增加间隔
max_records = 5000   # 限制最大记录数
```

**效果**: 每 20 秒或攒够 200 条才存储，最多保留 5000 条

### 场景 3: 禁用 SensorData 存储

```toml
[sensor_storage]
enabled = false
```

**效果**: 只存储 ProcessedData，节省存储空间

## 查看存储日志

### 启动程序并观察日志

```bash
./target/release/qt-rust-demo 2>&1 | grep -E "Saved.*records"
```

**预期输出**:

```
INFO Saved 10 records (seq <= 3321372)          # ProcessedData (每 1 秒)
INFO Saved 10 records (seq <= 3321382)
INFO Saved 10 records (seq <= 3321392)
...
INFO Saved 100 sensor data records              # SensorData (每 10 秒)
INFO Saved 10 records (seq <= 3321492)
...
```

### 只看 SensorData 存储

```bash
./target/release/qt-rust-demo 2>&1 | grep "sensor data records"
```

## 数据库表结构

### ProcessedData 表

```sql
CREATE TABLE runtime_data (
    id INTEGER PRIMARY KEY,
    sequence_number INTEGER,
    current_load REAL,
    rated_load REAL,
    working_radius REAL,
    boom_angle REAL,
    boom_length REAL,
    moment_percentage REAL,
    is_danger INTEGER,
    timestamp TEXT
);
```

### SensorData 表

```sql
CREATE TABLE sensor_data (
    id INTEGER PRIMARY KEY,
    ad1_load REAL,
    ad2_radius REAL,
    ad3_angle REAL,
    timestamp TEXT
);
```

## 常见问题

### Q: 为什么看不到 SensorData 存储日志？

A: 可能原因：
1. 还没攒够 100 条（需要 10 秒）
2. 还没到 60 秒间隔
3. `enabled = false` 被禁用了
4. 日志级别过滤了 INFO 日志

### Q: 两个存储管道会冲突吗？

A: 不会，它们：
- 使用不同的数据库表
- 使用不同的事件通道
- 独立运行，互不影响

### Q: 可以只启用一个吗？

A: 可以：
- 禁用 SensorData: `sensor_storage.enabled = false`
- 禁用 ProcessedData: 不推荐（会影响报警功能）

### Q: batch_size = 100 是否太大？

A: 取决于需求：
- **优点**: 减少数据库写入次数，提升性能
- **缺点**: 数据延迟最多 10 秒（或 60 秒）
- **建议**: 如果需要更实时的原始数据，改为 50 或 20

## 性能对比

| 指标 | ProcessedData | SensorData |
|------|--------------|-----------|
| 存储频率 | 每秒 1 次 | 每 10-60 秒 1 次 |
| 每次条数 | 10 条 | 100 条 |
| 数据大小 | ~200 字节/条 | ~50 字节/条 |
| 磁盘写入 | 2KB/秒 | 5KB/10秒 |
| CPU 占用 | ~1% | ~0.1% |

## 总结

- ✅ **ProcessedData**: 高频、实时、包含计算结果
- ✅ **SensorData**: 低频、原始、用于备份分析
- ✅ 两者独立运行，互不影响
- ✅ 可根据需求调整 batch_size 和 interval

---

**更新日期**: 2024-XX-XX  
**相关配置**: `config/pipeline_config.toml`
