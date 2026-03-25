# 管道配置系统使用指南

## 概述

管道配置系统允许你通过配置文件调整数据采集和存储管道的运行参数，无需修改代码即可优化系统性能。

## 配置文件位置

```
config/pipeline_config.toml
```

如果配置文件不存在，系统将使用默认配置。你可以从示例文件开始：

```bash
cp config/pipeline_config.toml.example config/pipeline_config.toml
```

## 配置项说明

### 1. 数据采集管道配置 [collection]

```toml
[collection]
interval_ms = 100        # 采集间隔（毫秒）
buffer_size = 1000       # 缓冲区大小
use_simulator = true     # 是否使用模拟传感器
```

**参数说明**:
- `interval_ms`: 从传感器读取数据的频率，建议 50-500ms
- `buffer_size`: 共享缓冲区可存储的最大数据条数
- `use_simulator`: true=使用模拟数据，false=使用真实传感器

**性能影响**:
- 间隔越小，数据采集越频繁，CPU 使用率越高
- 缓冲区越大，可以应对更长时间的存储延迟

### 2. 存储管道配置 [storage]

```toml
[storage]
interval_ms = 1000       # 存储间隔（毫秒）
batch_size = 10          # 批量存储大小
max_retries = 3          # 失败重试次数
retry_delay_ms = 100     # 重试延迟（毫秒）
max_queue_size = 1000    # 存储队列最大容量
```

**参数说明**:
- `interval_ms`: 批量存储运行数据的频率
- `batch_size`: 每次批量存储的最大数据条数
- `max_retries`: 存储失败时的最大重试次数
- `retry_delay_ms`: 重试前的等待时间
- `max_queue_size`: 存储队列可缓存的最大数据条数

**性能影响**:
- 存储间隔越大，批量大小越大，数据库写入次数越少
- 队列越大，可以应对更长时间的数据库延迟

### 3. 数据库配置 [database]

```toml
[database]
path = "crane_data.db"   # 数据库文件路径
enable_wal = true        # 是否启用 WAL 模式
pool_size = 5            # 连接池大小
```

**参数说明**:
- `path`: SQLite 数据库文件的存储路径
- `enable_wal`: Write-Ahead Logging 模式，提高并发性能
- `pool_size`: 数据库连接池的最大连接数

### 4. 传感器模拟器配置 [simulator]

```toml
[simulator.weight]
amplitude = 5.0          # 振幅（吨）
frequency = 0.5          # 频率（Hz）
offset = 15.0            # 偏移量（吨）
noise_level = 0.1        # 噪声水平

[simulator.radius]
amplitude = 3.0
frequency = 0.3
offset = 8.0
noise_level = 0.05

[simulator.angle]
amplitude = 10.0
frequency = 0.2
offset = 60.0
noise_level = 0.5
```

**参数说明**:
- `amplitude`: 正弦波振幅
- `frequency`: 正弦波频率
- `offset`: 基准值（偏移量）
- `noise_level`: 随机噪声水平（0-1）

### 5. 性能监控配置 [monitoring]

```toml
[monitoring]
enable = true            # 是否启用性能监控
stats_interval_sec = 5   # 监控统计间隔（秒）
verbose = false          # 是否打印详细日志
```

## 配置验证规则

系统会自动验证配置参数，确保合理性：

1. 采集间隔必须 >= 50ms
2. 存储间隔应 >= 采集间隔
3. 批量大小必须 > 0
4. 缓冲区大小必须 > 0
5. 队列大小必须 > 0
6. 数据库路径不能为空

如果验证失败，系统将拒绝加载配置并使用默认值。

## 使用场景示例

### 场景 1: 高频采集 + 快速存储

适用于需要高精度数据记录的场景。

```toml
[collection]
interval_ms = 50
buffer_size = 2000

[storage]
interval_ms = 500
batch_size = 20
```

**特点**: 数据采集频率高（20Hz），存储频率快（2Hz），数据延迟小。

### 场景 2: 低频采集 + 慢速存储（节省资源）

适用于资源受限的嵌入式设备。

```toml
[collection]
interval_ms = 500
buffer_size = 500

[storage]
interval_ms = 5000
batch_size = 50
```

**特点**: 数据采集频率低（2Hz），存储频率慢（0.2Hz），CPU 和磁盘使用率低。

### 场景 3: 生产环境配置

适用于实际部署的生产环境。

```toml
[collection]
interval_ms = 100
buffer_size = 1000
use_simulator = false

[storage]
interval_ms = 1000
batch_size = 10

[database]
path = "/var/lib/crane/crane_data.db"
enable_wal = true
pool_size = 10

[monitoring]
enable = true
verbose = false
```

**特点**: 使用真实传感器，数据库路径指向系统目录，启用性能监控。

## 运行时加载配置

### 方法 1: 自动加载

程序启动时会自动尝试加载 `config/pipeline_config.toml`：

```rust
use qt_rust_demo::config::PipelineConfig;

let config = PipelineConfig::load();
```

如果文件不存在或解析失败，将使用默认配置。

### 方法 2: 手动加载

指定配置文件路径：

```rust
let config = PipelineConfig::from_file("custom_config.toml")?;
```

### 方法 3: 使用默认配置

```rust
let config = PipelineConfig::default();
```

## 配置应用示例

### 在管道示例中使用

```rust
// examples/full_pipeline_demo.rs

use qt_rust_demo::config::PipelineConfig;

#[tokio::main]
async fn main() -> Result<(), String> {
    // 加载配置
    let config = PipelineConfig::load();
    
    println!("配置参数:");
    println!("  - 采集间隔: {}ms", config.collection.interval_ms);
    println!("  - 存储间隔: {}ms", config.storage.interval_ms);
    println!("  - 批量大小: {}", config.storage.batch_size);
    
    // 创建存储管道配置
    let storage_config = StoragePipelineConfig::from_pipeline_config(&config.storage);
    
    // 创建存储管道
    let mut storage_pipeline = StoragePipeline::new(
        storage_config,
        storage_repo,
        shared_buffer,
    )?;
    
    // 使用配置的采集间隔
    let collection_interval = Duration::from_millis(config.collection.interval_ms);
    
    Ok(())
}
```

## 测试配置

运行管道示例测试配置：

```bash
# 使用默认配置
cargo run --example full_pipeline_demo --features no-qt

# 修改配置后重新运行
vim config/pipeline_config.toml
cargo run --example full_pipeline_demo --features no-qt
```

## 性能调优建议

### 1. 采集间隔调优

- **高精度场景**: 50-100ms（10-20Hz）
- **标准场景**: 100-200ms（5-10Hz）
- **省电场景**: 500-1000ms（1-2Hz）

### 2. 存储间隔调优

- **实时性要求高**: 500-1000ms
- **标准场景**: 1000-2000ms
- **批量处理**: 5000-10000ms

### 3. 批量大小调优

- **小批量**: 5-10 条（低延迟）
- **中批量**: 10-50 条（平衡）
- **大批量**: 50-100 条（高吞吐）

### 4. 缓冲区大小调优

计算公式：
```
缓冲区大小 >= (存储间隔 / 采集间隔) × 批量大小 × 2
```

示例：
- 采集间隔 100ms，存储间隔 1000ms，批量大小 10
- 缓冲区大小 >= (1000/100) × 10 × 2 = 200

建议设置为计算值的 2-5 倍，以应对突发情况。

## 故障排查

### 问题 1: 配置文件加载失败

**现象**: 控制台显示 "Failed to load config"

**解决方案**:
1. 检查配置文件是否存在
2. 检查 TOML 语法是否正确
3. 查看详细错误信息

### 问题 2: 配置验证失败

**现象**: 控制台显示 "Failed to parse config file"

**解决方案**:
1. 检查参数值是否在合理范围内
2. 确保采集间隔 >= 50ms
3. 确保存储间隔 >= 采集间隔

### 问题 3: 数据丢失

**现象**: 部分数据未存储到数据库

**解决方案**:
1. 增大缓冲区大小
2. 增大存储队列大小
3. 减小批量大小，提高存储频率

### 问题 4: CPU 使用率过高

**现象**: 系统 CPU 占用率高

**解决方案**:
1. 增大采集间隔
2. 增大存储间隔
3. 增大批量大小

## 最佳实践

1. **开发环境**: 使用模拟器，高频采集，快速存储
2. **测试环境**: 使用真实传感器，标准配置
3. **生产环境**: 根据实际需求调优，启用监控
4. **修改前备份**: 修改配置前备份原文件
5. **逐步调整**: 每次只调整一个参数，观察效果
6. **监控性能**: 启用性能监控，记录关键指标

## 相关文档

- [三后台管道架构](THREE_BACKEND_PIPELINE_ARCHITECTURE.md)
- [管道集成指南](PIPELINE_INTEGRATION_GUIDE.md)
- [存储管道设计](STORAGE_PIPELINE_DESIGN.md)

---

**更新日期**: 2026-03-24
**版本**: 1.0.0
