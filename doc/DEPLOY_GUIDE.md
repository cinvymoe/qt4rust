# 设备部署指南

本指南说明如何将编译好的 Qt Rust 应用部署到 ARM32 设备并使用 Framebuffer 显示。

## 前置条件

1. 设备已连接并启用 ADB 调试
2. 设备具有 `/dev/fb0` framebuffer 设备
3. 应用已成功交叉编译

## 部署步骤

### 1. 检查设备连接

```bash
adb devices
```

应该看到设备列表，状态为 `device`。

### 2. 检查 Framebuffer 配置（可选但推荐）

```bash
./check-framebuffer.sh
```

这将显示：
- Framebuffer 设备信息
- 屏幕分辨率和色深
- 输入设备列表
- 访问权限

### 3. 部署应用

```bash
./deploy-to-device.sh
```

脚本会自动：
- 推送编译好的二进制文件
- 推送所有必需的 Qt6 共享库
- 推送 QML 资源文件
- 推送 Qt 插件
- 创建启动脚本

## 在设备上运行

### 方法 1: 使用启动脚本（推荐）

```bash
adb shell "cd /data/local/tmp/qt-rust-demo && ./run.sh"
```

### 方法 2: 手动设置环境变量

```bash
adb shell
cd /data/local/tmp/qt-rust-demo
source ./config.sh
./qt-rust-demo
```

### 方法 3: 直接运行（需要手动设置环境）

```bash
adb shell
cd /data/local/tmp/qt-rust-demo
export LD_LIBRARY_PATH=/data/local/tmp/qt-rust-demo/lib:$LD_LIBRARY_PATH
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0
./qt-rust-demo
```

## 环境变量说明

### 必需的环境变量

- `LD_LIBRARY_PATH`: 指向共享库目录
- `QT_QPA_PLATFORM`: 设置为 `linuxfb:fb=/dev/fb0` 使用 framebuffer

### 可选的环境变量

#### Framebuffer 配置

```bash
# 指定屏幕尺寸（如果自动检测失败）
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0:size=800x480

# 指定物理尺寸（毫米）
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0:mmSize=154x86

# 强制全屏
export QT_QPA_FB_FORCE_FULLSCREEN=1
```

#### 输入设备配置

```bash
# 启用输入设备
export QT_QPA_FB_DISABLE_INPUT=0

# 指定触摸屏设备
export QT_QPA_EVDEV_TOUCHSCREEN_PARAMETERS=/dev/input/event0

# 指定鼠标设备
export QT_QPA_EVDEV_MOUSE_PARAMETERS=/dev/input/event1

# 指定键盘设备
export QT_QPA_EVDEV_KEYBOARD_PARAMETERS=/dev/input/event2
```

#### 调试选项

```bash
# 启用 Qt 插件调试
export QT_DEBUG_PLUGINS=1

# 启用 QPA 调试日志
export QT_LOGGING_RULES="qt.qpa.*=true"

# 显示所有 Qt 日志
export QT_LOGGING_RULES="*=true"
```

#### 性能优化

```bash
# 使用软件渲染（如果硬件加速有问题）
export QT_QUICK_BACKEND=software

# 设置 DPI
export QT_FONT_DPI=96
```

## 常见问题

### 1. 屏幕无显示

**检查 framebuffer 权限：**
```bash
adb shell "ls -l /dev/fb0"
```

如果没有写权限，尝试：
```bash
adb shell "chmod 666 /dev/fb0"
```

**检查 framebuffer 是否被占用：**
某些设备的 framebuffer 可能被其他进程占用。

### 2. 分辨率不正确

手动指定分辨率：
```bash
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0:size=1920x1080
```

查看实际分辨率：
```bash
adb shell "cat /sys/class/graphics/fb0/virtual_size"
```

### 3. 触摸/鼠标不工作

查找输入设备：
```bash
adb shell "getevent -p"
```

然后指定正确的设备：
```bash
export QT_QPA_EVDEV_TOUCHSCREEN_PARAMETERS=/dev/input/eventX
```

### 4. 库加载失败

检查库是否正确推送：
```bash
adb shell "ls -l /data/local/tmp/qt-rust-demo/lib/"
```

检查库依赖：
```bash
adb shell "cd /data/local/tmp/qt-rust-demo && LD_LIBRARY_PATH=./lib ldd ./qt-rust-demo"
```

### 5. 权限问题

如果遇到权限错误，可能需要 root 权限：
```bash
adb root
adb remount
```

## 查看日志

### 实时查看应用输出

```bash
adb shell "cd /data/local/tmp/qt-rust-demo && ./run.sh" 2>&1 | tee app.log
```

### 查看系统日志

```bash
adb logcat | grep -i qt
```

## 性能监控

### 检查 CPU 使用率

```bash
adb shell "top -n 1 | grep qt-rust-demo"
```

### 检查内存使用

```bash
adb shell "dumpsys meminfo | grep qt-rust-demo"
```

## 清理

删除部署的文件：
```bash
adb shell "rm -rf /data/local/tmp/qt-rust-demo"
```

## 目录结构

部署后设备上的目录结构：

```
/data/local/tmp/qt-rust-demo/
├── qt-rust-demo          # 主程序
├── run.sh                # 启动脚本
├── config.sh             # 配置文件
├── lib/                  # 共享库
│   ├── libQt6Core.so.6
│   ├── libQt6Gui.so.6
│   ├── libQt6Qml.so.6
│   ├── libQt6Quick.so.6
│   └── ...
├── qml/                  # QML 资源
│   └── main.qml
└── plugins/              # Qt 插件
    └── platforms/
        └── libqlinuxfb.so
```

## 下一步

- 根据实际设备调整 `device-config.sh` 中的配置
- 测试触摸输入和显示效果
- 优化性能和内存使用
- 添加自动启动脚本（如果需要）
