# Qt Rust Demo

基于 Rust 语言和 Qt 框架的演示应用程序，使用 QML 进行 UI 开发，目标部署平台为 Linux ARM32/ARM64 设备。

## 项目概述

本项目展示了如何使用 Rust 与 Qt 6.x 框架集成，通过 cxx-qt 库实现 Rust 后端与 QML 前端的双向数据绑定和事件处理。应用程序支持交叉编译到 ARM32 和 ARM64 Linux 平台。

**详细文档**：
- [ARM64 支持文档](docs/arm64-support.md) - ARM64 交叉编译和部署指南

## 配置管理

### 配置文件说明

系统使用外部配置文件管理起重机参数，支持运行时重新加载，无需重新编译代码。配置文件位于 `config/` 目录：

| 配置文件 | 格式 | 用途 |
|---------|------|------|
| `config/sensor_calibration.toml` | TOML | 传感器标定参数（AD 值转物理值） |
| `config/rated_load_table.csv` | CSV | 额定载荷表（幅度与载荷对应关系） |

**首次运行时**，如果配置文件不存在，系统会自动生成包含默认值的配置文件。

### 配置参数说明

#### 传感器标定参数 (sensor_calibration.toml)

传感器标定使用**两点标定法**进行线性转换，将 AD 采集值转换为实际物理量：

**转换公式**：
```
物理值 = 零点物理值 + (AD值 - 零点AD) × (放大物理值 - 零点物理值) / (放大AD - 零点AD)
```

**重量传感器标定**：
- `weight_zero_ad`: 零点 AD 值（默认：0.0）
- `weight_zero_value`: 零点物理值，单位：吨（默认：0.0）
- `weight_scale_ad`: 放大 AD 值（默认：4095.0，12位 AD 满量程）
- `weight_scale_value`: 放大物理值，单位：吨（默认：50.0）

**角度传感器标定**：
- `angle_zero_ad`: 零点 AD 值（默认：0.0）
- `angle_zero_value`: 零点物理值，单位：度（默认：0.0）
- `angle_scale_ad`: 放大 AD 值（默认：4095.0）
- `angle_scale_value`: 放大物理值，单位：度（默认：90.0）

**半径传感器标定**：
- `radius_zero_ad`: 零点 AD 值（默认：0.0）
- `radius_zero_value`: 零点物理值，单位：米（默认：0.0）
- `radius_scale_ad`: 放大 AD 值（默认：4095.0）
- `radius_scale_value`: 放大物理值，单位：米（默认：20.0）

**预警和报警阈值**：
- `angle_warning_value`: 角度预警值，单位：度（默认：75.0）
- `angle_alarm_value`: 角度报警值，单位：度（默认：85.0）
- `moment_warning_percentage`: 力矩预警百分比（默认：90.0%）
- `moment_alarm_percentage`: 力矩报警百分比（默认：100.0%）

#### 额定载荷表 (rated_load_table.csv)

额定载荷表定义了不同工作半径下的额定载荷值，系统使用**阶梯查找**方式查询：
- 找到第一个 >= 当前半径的表项，返回其额定载荷
- 如果当前半径大于所有表项，返回最后一项的额定载荷

**CSV 格式**：
```csv
# moment_warning_threshold,85.0
# moment_alarm_threshold,95.0
radius_m,rated_load_ton
3.0,50.0
5.0,40.0
8.0,30.0
...
```

**注意事项**：
- 表项必须按半径升序排列
- 半径单位：米（m）
- 额定载荷单位：吨（ton）
- 所有数值必须大于 0

### 配置重载说明

系统支持**运行时重新加载配置**，无需重启应用程序：

#### 方法 1：通过 QML 界面（推荐）

1. 修改配置文件（`config/sensor_calibration.toml` 或 `config/rated_load_table.csv`）
2. 在应用程序中打开"设置"界面
3. 点击"重新加载配置"按钮
4. 系统会验证配置有效性并立即应用新配置

#### 方法 2：通过 API 调用

```rust
// 在 Rust 代码中重新加载配置
let config_manager = Arc::new(ConfigManager::new()?);
match config_manager.reload_config() {
    Ok(new_config) => println!("配置重载成功"),
    Err(e) => eprintln!("配置重载失败: {}", e),
}
```

#### 配置验证

重新加载配置时，系统会自动验证：
- 标定参数是否会导致除零错误
- 阈值是否在有效范围内（0-100%）
- 报警阈值是否大于等于预警阈值
- 载荷表是否按半径升序排列

**如果验证失败**，系统会保持当前配置不变，并返回详细的错误信息。

#### 配置生效时机

配置重载成功后，**下次数据处理时立即生效**，无需重启应用程序或重新启动数据采集。

## 依赖项

### Rust 工具链

- **Rust 版本**: 1.70.0 或更高（推荐使用 stable channel）
- **Cargo**: 随 Rust 工具链自动安装
- **目标平台支持**: armv7-unknown-linux-gnueabihf

安装 Rust 工具链：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

添加 ARM32 交叉编译目标：
```bash
rustup target add armv7-unknown-linux-gnueabihf
```

### Qt 库

- **Qt 版本**: 6.2 或更高（推荐 6.5+）
- **必需模块**: QtCore, QtGui, QtQml, QtQuick

#### Ubuntu/Debian 安装 Qt 6:
```bash
sudo apt update
sudo apt install qt6-base-dev qt6-declarative-dev qml6-module-qtquick-controls
```

#### 验证 Qt 安装:
```bash
qmake6 --version
# 或
qmake -version
```

### 交叉编译工具链

针对 ARM32 目标平台：

```bash
sudo apt install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
```

### 系统依赖

- **构建工具**: cmake, pkg-config, clang
- **开发库**: libclang-dev

```bash
sudo apt install cmake pkg-config clang libclang-dev
```

## 开发环境配置

### 1. 克隆项目

```bash
git clone <repository-url>
cd qt-rust-demo
```

### 2. 配置环境变量

根据您的 Qt 安装路径，可能需要设置以下环境变量：

```bash
export QT_QML_IMPORT_PATH=/usr/lib/qt6/qml
export QT_PLUGIN_PATH=/usr/lib/qt6/plugins
export LD_LIBRARY_PATH=/usr/lib/qt6/lib:$LD_LIBRARY_PATH
```

### 3. 本地构建（x86_64）

```bash
cargo build
cargo run
```

### 4. 运行测试

```bash
# 单元测试
cargo test

# 属性测试（release 模式，100+ 迭代）
cargo test --release

# 代码检查
cargo clippy

# 格式检查
cargo fmt --check
```

## 交叉编译配置

### ARM32 Linux 目标平台

#### 1. 安装交叉编译工具链

```bash
rustup target add armv7-unknown-linux-gnueabihf
sudo apt install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
```

#### 2. 配置 Qt 库路径

如果目标设备使用不同的 Qt 库路径，需要在 `.cargo/config.toml` 中配置：

```toml
[env]
PKG_CONFIG_SYSROOT_DIR = "/path/to/arm-sysroot"
PKG_CONFIG_PATH = "/path/to/arm-sysroot/usr/lib/pkgconfig"
```

#### 3. 交叉编译

```bash
cargo build --target armv7-unknown-linux-gnueabihf --release
```

#### 4. 验证生成的二进制文件

```bash
file target/armv7-unknown-linux-gnueabihf/release/qt-rust-demo
# 输出应显示: ELF 32-bit LSB executable, ARM, ...

arm-linux-gnueabihf-readelf -d target/armv7-unknown-linux-gnueabihf/release/qt-rust-demo
# 检查动态链接库依赖
```

### ARM64 Linux 目标平台

#### 1. 安装交叉编译工具链

```bash
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
```

#### 2. 交叉编译

```bash
cargo build --target aarch64-unknown-linux-gnu --release
```

#### 3. 验证生成的二进制文件

```bash
file target/aarch64-unknown-linux-gnu/release/qt-rust-demo
# 输出应显示: ELF 64-bit LSB executable, ARM aarch64, ...

aarch64-linux-gnu-readelf -d target/aarch64-unknown-linux-gnu/release/qt-rust-demo
# 检查动态链接库依赖
```

### 使用 Makefile 简化编译

```bash
# ARM32
make build

# ARM64
make build-arm64
```

## 部署到设备

### 使用 Makefile（推荐）

```bash
# ARM32 完整部署
make deploy

# ARM64 完整部署
make deploy-arm64

# 分步执行
make build-arm64    # 编译
make push-arm64     # 推送到设备
make run-arm64      # 运行
```

### ARM32 手动部署

#### 1. 复制二进制文件到目标设备

```bash
scp target/armv7-unknown-linux-gnueabihf/release/qt-rust-demo user@device-ip:/home/user/
```

### 2. 复制 QML 文件（如果未嵌入资源）

```bash
scp -r qml/ user@device-ip:/home/user/qt-rust-demo/
```

### 3. 在目标设备上运行

```bash
ssh user@device-ip
cd /home/user
./qt-rust-demo
```

### 4. 确保目标设备已安装 Qt 运行时

```bash
# 在 ARM32 设备上
sudo apt install qt6-base-runtime qt6-declarative-runtime
```

### ARM64 手动部署

ARM64 设备通常使用 Wayland 显示，需要额外的环境配置。详见 [ARM64 支持文档](docs/arm64-support.md)。

#### 使用部署脚本

```bash
# 设置架构并部署
ARCH=arm64 ./scripts/deploy-to-device.sh

# 运行
ARCH=arm64 ./scripts/run-on-device.sh
```

#### 关键环境变量

ARM64 Wayland 设备需要以下环境变量：

```bash
export QT_QPA_PLATFORM=wayland
export QT_QUICK_BACKEND=software
export WAYLAND_WAIT=1
export XDG_RUNTIME_DIR=/var/run
export WAYLAND_DISPLAY=wayland-0
```

## 传感器数据管线架构

sensor-core crate 实现了多源传感器数据的采集、聚合和存储管线。数据从多个独立来源流入，经过聚合后批量写入存储后端。

### 管线数据流

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Modbus      │  │  Simulator  │  │  Mock       │
│  Source      │  │  Source     │  │  Source      │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Sensor     │  │  Sensor     │  │  Sensor      │
│  Pipeline   │  │  Pipeline   │  │  Pipeline    │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                  │                  │
       └──────────┬───────┴──────────────────┘
                  │  mpsc channel
                  ▼
          ┌───────────────┐
          │  Aggregator   │
          │  Pipeline     │
          └───────┬───────┘
                  │  mpsc channel
                  ▼
          ┌───────────────┐
          │  Storage      │
          │  Pipeline     │
          └───────┬───────┘
                  │
                  ▼
          ┌───────────────┐
          │  Storage      │
          │  Repository   │
          │  (trait)      │
          └───────────────┘
```

### 核心组件

| 组件 | 职责 |
|------|------|
| `SensorPipelineManager` | 管线总管：注册数据源、配置聚合策略、设置存储后端、启停管线 |
| `SensorPipeline<S>` | 从 `SensorSource` 定时读取数据，带重试逻辑，通过 channel 发送 |
| `AggregatorPipeline` | 按 `AggregationStrategy` 合并多源数据，输出 `AggregatedSensorData` |
| `StoragePipeline` | 批量写入聚合数据，按大小或时间间隔刷盘，停机时最终刷盘 |
| `StorageRepository` (trait) | 存储后端抽象，可替换为 SQLite、PostgreSQL 或内存实现 |

### 聚合策略

| 策略 | 行为 |
|------|------|
| `Immediate` | 任一数据源上报即发出聚合数据 |
| `WaitAll(duration)` | 等待所有已注册数据源上报，超时后发出 |
| `PrimaryBackup { primary, backup }` | 以主数据源为准，主源不可用时切换到备用源 |

### 使用示例

```rust
use sensor_core::{
    AggregationStrategy, DataSourceId, PipelineConfig,
    SensorPipelineManager, StoragePipelineConfig,
};
use std::sync::Arc;
use std::time::Duration;

let mut manager = SensorPipelineManager::new();

// 注册数据源
manager.register_source(
    DataSourceId::Modbus,
    Arc::new(my_modbus_source),
    PipelineConfig {
        read_interval: Duration::from_millis(100),
        max_retries: 3,
        debug_logging: false,
    },
);

// 配置聚合策略
manager.set_aggregation_strategy(
    AggregationStrategy::WaitAll(Duration::from_millis(50))
);

// 配置存储
manager.set_storage_config(StoragePipelineConfig {
    storage_interval: Duration::from_secs(5),
    batch_size: 100,
    enable_compression: false,
});
manager.set_storage_repository(Arc::new(my_storage_impl));

// 启动所有管线
manager.start_all()?;

// ... 运行应用 ...

// 按序停机：传感器 → 存储 → 聚合器
manager.stop_all();
```

详细文档见 [crates/sensor-core/README.md](crates/sensor-core/README.md)。

## 项目结构

本项目采用 Cargo Workspace 结构，将核心业务逻辑和 Qt UI 分离：

```
qt-rust-demo/                           # Workspace 根目录
├── Cargo.toml                          # Workspace 配置
├── src/
│   └── lib.rs                          # 核心库（无 Qt 依赖）
├── crates/
│   ├── qt-app/                         # Qt GUI 应用 ✨
│   │   ├── Cargo.toml
│   │   ├── build.rs                    # cxx-qt 构建脚本
│   │   ├── qml/ -> ../../qml           # QML 文件（符号链接）
│   │   └── src/
│   │       ├── main.rs                 # 应用入口
│   │       ├── application.rs          # Qt 应用初始化
│   │       ├── monitoring_viewmodel.rs # 监控 ViewModel
│   │       └── ...
│   ├── sensor-core/                    # 传感器管线核心库
│   │   ├── Cargo.toml
│   │   ├── README.md                   # 管线架构文档
│   │   └── src/
│   │       ├── lib.rs                  # 公共 API 导出
│   │       ├── pipeline/               # 多源管线架构
│   │       │   ├── manager.rs          # SensorPipelineManager
│   │       │   ├── sensor_pipeline.rs  # 单源采集管线
│   │       │   ├── aggregator.rs       # 聚合管线 + 策略
│   │       │   ├── storage.rs          # 存储管线
│   │       │   ├── config.rs           # 管线配置
│   │       │   └── data_source.rs      # DataSourceId, SourceSensorData
│   │       ├── storage/                # 存储抽象
│   │       │   └── repository.rs       # StorageRepository trait
│   │       ├── sensors/                # 传感器实现
│   │       ├── calibration/            # 标定模块
│   │       ├── algorithms/             # AD 转换算法
│   │       ├── data/                   # 数据模型
│   │       ├── traits.rs               # SensorSource, SensorProvider
│   │       └── error.rs                # 错误类型
│   ├── cxx-qt-mvi-core/                # MVI 架构核心库
│   ├── sensor-simulator/               # 传感器模拟器
│   ├── qt-threading-utils/             # Qt 线程工具
│   └── crane-data-layer/               # 数据层抽象
├── qml/                                # QML 前端代码
│   ├── main.qml
│   ├── views/
│   └── components/
├── config/                             # 配置文件
│   ├── sensor_calibration.toml
│   └── rated_load_table.csv
└── examples/                           # 示例程序（无 Qt）
    ├── full_pipeline_demo.rs
    └── test_storage_interval.rs
```

### 架构优势

- **lib (qt-rust-demo)**: 核心业务逻辑，不依赖 Qt，可独立测试和复用
- **sensor-core**: 传感器管线核心，纯 Rust 异步实现，无 Qt 依赖
- **qt-app**: Qt GUI 应用，依赖 lib，只包含 UI 相关代码
- **分离的好处**:
  1. lib 和 sensor-core 可以在没有 Qt 环境的机器上编译和测试
  2. sensor-core 可以被其他项目复用（CLI 工具、Web 服务等）
  3. 职责清晰，易于维护

## 编译和运行

### 编译核心库（无 Qt）

```bash
# 编译 lib
cargo build --lib

# 运行示例程序
cargo run --example full_pipeline_demo
```

### 编译 Qt 应用

```bash
# 本地编译
cargo build -p qt-app

# 运行
cargo run -p qt-app

# 交叉编译到 ARM32
cargo build -p qt-app --target armv7-unknown-linux-gnueabihf --release
```

### 编译整个 Workspace

```bash
cargo build --workspace
```

## 性能指标

- **启动时间**: < 3 秒
- **内存占用**: < 100MB
- **UI 响应时间**: < 100ms（按钮点击到界面更新）

## 故障排除

### Qt 库未找到

如果遇到 "Qt libraries not found" 错误：

1. 确认 Qt 6 已正确安装
2. 设置 `QT_DIR` 环境变量指向 Qt 安装目录
3. 检查 `pkg-config --modversion Qt6Core` 是否返回版本号

### 交叉编译链接错误

如果交叉编译时遇到链接错误：

1. 确认 ARM 工具链已安装：`arm-linux-gnueabihf-gcc --version`
2. 检查 `.cargo/config.toml` 中的链接器配置
3. 确保目标设备的 sysroot 路径正确

### QML 文件加载失败

如果应用启动时 QML 加载失败：

1. 检查 QML 文件路径是否正确
2. 确认 `QT_QML_IMPORT_PATH` 环境变量已设置
3. 验证 QML 文件语法是否正确

## 许可证

本项目仅用于技术演示和学习目的。

## 贡献

欢迎提交 Issue 和 Pull Request。

## 参考资料

- [cxx-qt 文档](https://github.com/KDAB/cxx-qt)
- [Qt 6 文档](https://doc.qt.io/qt-6/)
- [Rust 交叉编译指南](https://rust-lang.github.io/rustup/cross-compilation.html)
