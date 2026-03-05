# Qt Rust Demo

基于 Rust 语言和 Qt 框架的演示应用程序，使用 QML 进行 UI 开发，目标部署平台为 Linux ARM32 设备。

## 项目概述

本项目展示了如何使用 Rust 与 Qt 6.x 框架集成，通过 cxx-qt 库实现 Rust 后端与 QML 前端的双向数据绑定和事件处理。应用程序支持交叉编译到 ARM32 Linux 平台。

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

## 部署到 ARM32 设备

### 1. 复制二进制文件到目标设备

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

## 项目结构

```
qt-rust-demo/
├── Cargo.toml              # Rust 项目配置和依赖
├── build.rs                # cxx-qt 构建脚本
├── .cargo/
│   └── config.toml         # 交叉编译配置
├── src/
│   ├── main.rs             # 应用程序入口点
│   ├── counter.rs          # Counter 业务对象（待实现）
│   └── application.rs      # Application 结构体（待实现）
├── qml/
│   └── main.qml            # QML 主界面
└── tests/                  # 集成测试目录

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
