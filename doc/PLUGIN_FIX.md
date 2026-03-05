# Qt 平台插件问题修复

## 问题

应用启动时出现错误：
```
qt.qpa.plugin: Could not find the Qt platform plugin "linuxfb" in "/data/local/tmp/qt-rust-demo/plugins"
This application failed to start because no Qt platform plugin could be initialized.
```

## 原因

Qt 应用需要平台插件来与底层图形系统交互。对于 framebuffer 输出，需要 `libqlinuxfb.so` 插件。

## 解决方案

### 方法 1: 运行插件推送脚本（推荐）

```bash
./push-plugins.sh
```

这会自动推送：
- linuxfb 平台插件
- 插件所需的依赖库
- 其他可选的平台插件

### 方法 2: 手动推送

```bash
# 创建插件目录
adb shell "mkdir -p /data/local/tmp/qt-rust-demo/plugins/platforms"

# 推送 linuxfb 插件
adb push /usr/lib/arm-linux-gnueabihf/qt6/plugins/platforms/libqlinuxfb.so \
    /data/local/tmp/qt-rust-demo/plugins/platforms/

# 推送插件依赖
adb push /usr/lib/arm-linux-gnueabihf/libudev.so.1 \
    /data/local/tmp/qt-rust-demo/lib/
adb push /usr/lib/arm-linux-gnueabihf/libmtdev.so.1 \
    /data/local/tmp/qt-rust-demo/lib/
adb push /usr/lib/arm-linux-gnueabihf/libts.so.0 \
    /data/local/tmp/qt-rust-demo/lib/
adb push /usr/lib/arm-linux-gnueabihf/libinput.so.10 \
    /data/local/tmp/qt-rust-demo/lib/
adb push /usr/lib/arm-linux-gnueabihf/libdrm.so.2 \
    /data/local/tmp/qt-rust-demo/lib/
```

### 方法 3: 重新运行完整部署

更新后的 `deploy-to-device.sh` 已包含插件推送：

```bash
./deploy-to-device.sh
```

## 插件依赖

linuxfb 插件需要以下库：

| 库 | 用途 |
|---|---|
| libudev.so.1 | 设备管理 |
| libmtdev.so.1 | 多点触控支持 |
| libts.so.0 | 触摸屏支持 |
| libinput.so.10 | 输入设备处理 |
| libdrm.so.2 | Direct Rendering Manager |
| libxkbcommon.so.0 | 键盘映射（已包含） |

## 验证

### 检查插件是否存在

```bash
adb shell "ls -l /data/local/tmp/qt-rust-demo/plugins/platforms/"
```

应该看到：
```
libqlinuxfb.so
```

### 检查环境变量

确保启动脚本设置了正确的环境变量：

```bash
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0
export QT_QPA_PLATFORM_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins
```

### 测试运行

```bash
adb shell "cd /data/local/tmp/qt-rust-demo && ./run.sh"
```

## 其他可用的平台插件

除了 linuxfb，还可以尝试其他平台插件：

### minimal
最小化插件，无图形输出，用于测试：
```bash
export QT_QPA_PLATFORM=minimal
```

### offscreen
离屏渲染，用于无头环境：
```bash
export QT_QPA_PLATFORM=offscreen
```

### eglfs
如果设备支持 EGL：
```bash
export QT_QPA_PLATFORM=eglfs
```

## 调试

### 启用插件调试

```bash
export QT_DEBUG_PLUGINS=1
export QT_LOGGING_RULES="qt.qpa.*=true"
./qt-rust-demo
```

这会显示详细的插件加载信息。

### 检查插件路径

```bash
# 在设备上
echo $QT_QPA_PLATFORM_PLUGIN_PATH
ls -la $QT_QPA_PLATFORM_PLUGIN_PATH/platforms/
```

### 检查插件依赖

```bash
# 在主机上
arm-linux-gnueabihf-readelf -d \
    /usr/lib/arm-linux-gnueabihf/qt6/plugins/platforms/libqlinuxfb.so \
    | grep NEEDED
```

## 常见问题

### 1. 插件加载失败

**症状：** "Could not load the Qt platform plugin"

**解决：**
- 检查插件文件是否存在
- 检查 `QT_QPA_PLATFORM_PLUGIN_PATH` 是否正确
- 检查插件依赖库是否都已推送

### 2. 权限问题

**症状：** Permission denied

**解决：**
```bash
adb shell "chmod 755 /data/local/tmp/qt-rust-demo/plugins/platforms/libqlinuxfb.so"
```

### 3. Framebuffer 访问失败

**症状：** "Could not open framebuffer device"

**解决：**
```bash
# 检查 framebuffer 权限
adb shell "ls -l /dev/fb0"

# 修改权限
adb shell "chmod 666 /dev/fb0"
```

### 4. 输入设备不工作

**症状：** 触摸或键盘无响应

**解决：**
```bash
# 启用输入
export QT_QPA_FB_DISABLE_INPUT=0

# 指定输入设备
export QT_QPA_EVDEV_TOUCHSCREEN_PARAMETERS=/dev/input/event0
```

## 完整的环境变量示例

```bash
#!/system/bin/sh
cd /data/local/tmp/qt-rust-demo

# 库路径
export LD_LIBRARY_PATH=/data/local/tmp/qt-rust-demo/lib:$LD_LIBRARY_PATH

# 平台配置
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0
export QT_QPA_PLATFORM_PLUGIN_PATH=/data/local/tmp/qt-rust-demo/plugins

# QML 路径
export QML2_IMPORT_PATH=/data/local/tmp/qt-rust-demo/qml

# 输入设备
export QT_QPA_FB_DISABLE_INPUT=0

# 调试（可选）
# export QT_DEBUG_PLUGINS=1
# export QT_LOGGING_RULES="qt.qpa.*=true"

./qt-rust-demo
```

## 相关文件

- `push-plugins.sh` - 插件推送脚本
- `deploy-to-device.sh` - 完整部署脚本（已更新）
- `device-config.sh` - 设备配置示例
- `run.sh` - 设备上的启动脚本
