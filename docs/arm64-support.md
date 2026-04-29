# ARM64 支持文档

本文档记录 Qt Rust Demo 项目的 ARM64 交叉编译和设备部署支持。

## 概述

项目支持两种目标架构：
- **ARM32** (默认): `armv7-unknown-linux-gnueabihf`
- **ARM64**: `aarch64-unknown-linux-gnu`

两种架构完全解耦，各自使用独立的库路径和插件，不会混用。

## 目标设备

- **平台**: RK3568 Buildroot
- **架构**: aarch64 (ARM64)
- **显示**: Weston Wayland compositor
- **GPU**: Mali Bifrost G52 (libmali-bifrost-g52-g13p0-wayland-gbm.so)
- **系统 Qt**: Qt 5.15.8

## 快速开始

```bash
# 完整部署 ARM64（编译 + 推送 + 运行）
make deploy-arm64

# 或分步执行
make build-arm64    # 编译
make push-arm64     # 推送到设备
make run-arm64      # 运行
```

## 命令参考

### Make 命令

| 命令 | 说明 |
|------|------|
| `make build-arm64` | 交叉编译 ARM64 版本 |
| `make push-arm64` | 推送 ARM64 应用和依赖到设备 |
| `make push-arm64-no-plugins` | 推送 ARM64 应用（跳过 Qt 插件和共享库） |
| `make run-arm64` | 在设备上运行 ARM64 应用 |
| `make deploy-arm64` | 完整部署：编译 + 推送 + 运行 |
| `make stop` | 停止设备上的应用 |

### 脚本命令

```bash
# 部署
ARCH=arm64 ./scripts/deploy-to-device.sh

# 运行
ARCH=arm64 ./scripts/run-on-device.sh

# 收集库
ARCH=arm64 ./scripts/collect-libs.sh

# 推送插件
ARCH=arm64 ./scripts/push-plugins.sh
```

## 环境配置

### ARM64 运行环境变量

```bash
export LD_LIBRARY_PATH=/data/local/tmp/qt-rust-demo/lib:/usr/lib
export QT_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins
export XDG_RUNTIME_DIR=/var/run
export WAYLAND_DISPLAY=wayland-0
export WAYLAND_WAIT=1
export QT_QPA_PLATFORM=wayland
export QML2_IMPORT_PATH=/data/local/tmp/qt-rust-demo/qml_modules
export QT_QUICK_BACKEND=software
```

### 关键配置说明

| 变量 | 值 | 说明 |
|------|-----|------|
| `QT_QPA_PLATFORM` | `wayland` | 使用 Wayland 显示平台 |
| `QT_QUICK_BACKEND` | `software` | 软件渲染（绕过 EGL 问题） |
| `WAYLAND_WAIT` | `1` | 等待 Wayland compositor 就绪 |

## 技术细节

### 为什么使用软件渲染？

设备的硬件 OpenGL/EGL 配置存在兼容性问题：

1. **EGL 存根问题**: 设备的 `/usr/lib/libEGL.so.1` 是 5KB 存根文件，不是真正的 EGL 实现
2. **Mali GPU 驱动**: 真正的 EGL 在 `libmali.so.1.9.0` 中
3. **Qt5 vs Qt6**: 
   - Qt5 的 `wayland-egl` 插件直接链接 Mali 库，硬件加速正常
   - Qt6 的 `wayland-egl` 插件使用标准 EGL 接口，与设备不兼容

**解决方案**: 使用 `QT_QUICK_BACKEND=software` 启用软件渲染，绕过 EGL 依赖。

### 架构解耦设计

所有部署脚本都支持 `ARCH` 环境变量，确保架构完全解耦：

| 脚本 | 架构判断 | 库路径 |
|------|----------|--------|
| `deploy-to-device.sh` | ✅ | 根据 ARCH 选择二进制和插件路径 |
| `collect-libs.sh` | ✅ | 根据 ARCH 收集对应架构的库 |
| `push-plugins.sh` | ✅ | 根据 ARCH 推送对应架构的插件 |
| `run-on-device.sh` | ✅ | 根据 ARCH 设置运行环境 |

### 库路径映射

| 架构 | 库基础路径 | Qt 插件路径 |
|------|------------|-------------|
| ARM32 | `/usr/lib/arm-linux-gnueabihf` | `/usr/lib/arm-linux-gnueabihf/qt6/plugins` |
| ARM64 | `/usr/lib/aarch64-linux-gnu` | `/usr/lib/aarch64-linux-gnu/qt6/plugins` |

## 部署内容

ARM64 部署时推送以下内容：

```
/data/local/tmp/qt-rust-demo/
├── qt-rust-demo          # 应用二进制
├── lib/                  # 共享库
│   ├── libQt6Core.so.6
│   ├── libQt6Gui.so.6
│   ├── libQt6Qml.so.6
│   └── ...
├── plugins/              # Qt 插件
│   ├── platforms/        # 平台插件
│   │   └── libqwayland-generic.so
│   ├── imageformats/     # 图像格式插件
│   │   └── libqsvg.so
│   ├── wayland-shell-integration/
│   │   └── libxdg-shell.so
│   └── platforminputcontexts/
│       └── libqtvirtualkeyboardplugin.so
├── qml_modules/          # QML 模块
│   ├── QtQuick/
│   ├── QtQml/
│   └── ...
├── qml/                  # 应用 QML 文件
├── config/               # 配置文件
└── fonts/                # 字体文件
```

## 故障排除

### 问题：应用启动后无显示

**原因**: Wayland compositor 未运行或环境变量未设置

**解决**:
```bash
# 检查 Weston 是否运行
adb shell "ps aux | grep weston"

# 检查 Wayland socket
adb shell "ls -la /var/run/wayland-*"
```

### 问题：SVG 图标不显示

**原因**: 缺少 SVG 图像格式插件

**解决**:
```bash
# 推送 SVG 插件
adb push /usr/lib/aarch64-linux-gnu/qt6/plugins/imageformats/libqsvg.so \
    /data/local/tmp/qt-rust-demo/plugins/imageformats/
```

### 问题：QML 模块加载失败

**原因**: `QML2_IMPORT_PATH` 未正确设置

**解决**:
```bash
adb shell "ls -la /data/local/tmp/qt-rust-demo/qml_modules/QtQuick/"
```

### 问题：库加载失败

**原因**: `LD_LIBRARY_PATH` 未包含设备系统库路径

**解决**: 确保 `LD_LIBRARY_PATH` 包含 `/usr/lib`:
```bash
export LD_LIBRARY_PATH=/data/local/tmp/qt-rust-demo/lib:/usr/lib
```

## 开发历史

### 2024-04-29 ARM64 支持完成

1. **交叉编译配置**: 添加 `aarch64-unknown-linux-gnu` 目标
2. **库收集**: `collect-libs.sh` 支持 ARCH 变量
3. **部署脚本**: `deploy-to-device.sh` 支持 ARM64 路径
4. **运行脚本**: `run-on-device.sh` 支持 Wayland + 软件渲染
5. **架构解耦修复**: 
   - 修复 `deploy-to-device.sh` 调用 `collect-libs.sh` 时未传递 ARCH
   - 修复 `push-plugins.sh` 硬编码 ARM32 路径
6. **SVG 支持**: 推送 Qt6 SVG 图像格式插件

### 关键发现

- 设备使用 Qt5 5.15.8，Qt6 需要完整部署
- `WAYLAND_WAIT=1` 是 Wayland 应用的关键环境变量
- 软件渲染是解决 EGL 兼容性问题的可靠方案

## 参考链接

- [Qt6 Wayland 平台插件](https://doc.qt.io/qt-6/qpa-wayland.html)
- [Qt6 软件渲染](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph-renderer.html)
- [Mali GPU 驱动](https://developer.arm.com/tools-and-software/graphics-and-gaming/mali-drivers)
