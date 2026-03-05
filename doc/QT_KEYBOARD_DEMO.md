# Qt Virtual Keyboard Demo

这是一个使用 Qt 官方 Qt Virtual Keyboard 模块的虚拟键盘演示程序。

## 功能特性

- ✅ 使用 Qt 官方 QtQuick.VirtualKeyboard 模块
- ✅ 支持多种输入类型（文本、密码、邮箱、数字、电话、URL）
- ✅ 自动弹出/隐藏键盘动画
- ✅ 响应式布局，支持滚动
- ✅ 多行文本输入支持
- ✅ 支持 ADB 设备部署

## 文件结构

```
qml/qt-keyboard-demo.qml              # Qt Virtual Keyboard Demo QML 文件
scripts/deploy-qt-keyboard-demo.sh    # ADB 设备部署脚本
scripts/run-qt-keyboard-on-device.sh  # ADB 设备运行脚本
scripts/run-qt-keyboard-demo.sh       # 本地 Bash 启动脚本
scripts/preview_qt_keyboard.py        # Python 预览脚本
```

## 运行方式

### 方式 1: 在 ADB 设备上运行（ARM 设备）

#### 步骤 1: 部署到设备

```bash
./scripts/deploy-qt-keyboard-demo.sh
```

这个脚本会：
- 检查 ADB 设备连接
- 推送 QML 文件到设备
- 推送必要的 Qt 库和插件
- 推送 Qt Virtual Keyboard 插件
- 创建启动脚本

#### 步骤 2: 运行应用

```bash
./scripts/run-qt-keyboard-on-device.sh
```

或者直接在设备上运行：

```bash
adb shell "sh /data/local/tmp/qt-keyboard-demo/run.sh"
```

#### 手动运行（调试用）

```bash
adb shell
cd /data/local/tmp/qt-keyboard-demo
export LD_LIBRARY_PATH=./lib:$LD_LIBRARY_PATH
export QT_IM_MODULE=qtvirtualkeyboard
export QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0
export QT_PLUGIN_PATH=./plugins
export QML2_IMPORT_PATH=./qml_modules
qml6 qml/qt-keyboard-demo.qml
```

### 方式 2: 使用 Bash 脚本（本地 Linux）

```bash
./scripts/run-qt-keyboard-demo.sh
```

### 方式 3: 使用 Python 预览（推荐用于开发）

```bash
# 首先安装 PySide6
pip install PySide6

# 运行预览
python3 scripts/preview_qt_keyboard.py
```

### 方式 4: 直接使用 qml 命令

```bash
export QT_IM_MODULE=qtvirtualkeyboard
export QT_VIRTUALKEYBOARD_DESKTOP_DISABLE=0
qml6 qml/qt-keyboard-demo.qml
```

## 环境要求

### Qt 模块依赖

需要安装以下 Qt 模块：

- QtQuick
- QtQuick.Controls
- QtQuick.Layouts
- QtQuick.VirtualKeyboard

### 安装 Qt Virtual Keyboard

#### Ubuntu/Debian

```bash
sudo apt-get install qml6-module-qtvirtualkeyboard
```

#### ARM 交叉编译环境

```bash
sudo apt-get install qml6-module-qtvirtualkeyboard:armhf
```

#### 从源码编译

如果系统包管理器中没有，需要从 Qt 源码编译：

```bash
git clone https://code.qt.io/qt/qtvirtualkeyboard.git
cd qtvirtualkeyboard
git checkout 6.4.2  # 使用你的 Qt 版本
mkdir build && cd build
cmake .. -DCMAKE_INSTALL_PREFIX=/usr/local/qt6
make -j$(nproc)
sudo make install
```

## 环境变量说明

- `QT_IM_MODULE=qtvirtualkeyboard`: 启用 Qt Virtual Keyboard 输入法模块
- `QT_VIRTUALKEYBOARD_DESKTOP_DISABLE=0`: 在桌面环境中启用虚拟键盘（默认桌面环境禁用）
- `QT_QPA_PLATFORM=linuxfb:fb=/dev/fb0`: 使用 Linux Framebuffer（嵌入式设备）
- `QML2_IMPORT_PATH`: QML 模块搜索路径
- `QT_PLUGIN_PATH`: Qt 插件搜索路径

## 输入类型说明

Demo 中展示了多种输入类型，每种类型会显示不同的键盘布局：

| 输入类型 | InputMethodHints | 键盘布局 |
|---------|------------------|---------|
| 普通文本 | 默认 | 完整键盘 |
| 密码 | ImhSensitiveData + ImhNoPredictiveText | 无预测文本 |
| 邮箱 | ImhEmailCharactersOnly | 包含 @ 和 . |
| 数字 | ImhDigitsOnly | 数字键盘 |
| 电话 | ImhDialableCharactersOnly | 拨号键盘 |
| URL | ImhUrlCharactersOnly | 包含 :// 和 . |
| 多行文本 | 默认 | 完整键盘 + 换行 |

## 键盘特性

Qt Virtual Keyboard 提供以下特性：

1. **多语言支持**: 支持 30+ 种语言
2. **手写识别**: 支持中文、日文等手写输入
3. **自动完成**: 智能预测和自动完成
4. **主题定制**: 可自定义键盘外观
5. **布局切换**: 根据输入类型自动切换布局

## 自定义配置

### 启用特定语言

在 QML 中设置：

```qml
InputPanel {
    // 只启用英文和中文
    VirtualKeyboardSettings.activeLocales: ["en_US", "zh_CN"]
}
```

### 自定义键盘样式

创建自定义样式文件并设置环境变量：

```bash
export QT_VIRTUALKEYBOARD_STYLE=retro
```

可用样式：
- default
- retro

## ADB 设备部署说明

### 设备要求

- Android 或 ARM Linux 设备
- 支持 Framebuffer (/dev/fb0)
- 已启用 ADB 调试
- 有足够的存储空间（约 50MB）

### 部署目录结构

```
/data/local/tmp/qt-keyboard-demo/
├── qml/
│   └── qt-keyboard-demo.qml
├── lib/
│   ├── libQt6Core.so.6
│   ├── libQt6Gui.so.6
│   ├── libQt6Qml.so.6
│   ├── libQt6Quick.so.6
│   ├── libQt6VirtualKeyboard.so.6
│   └── ...
├── plugins/
│   ├── platforms/
│   │   └── libqlinuxfb.so
│   └── platforminputcontexts/
│       └── libqtvirtualkeyboardplugin.so
├── qml_modules/
│   ├── QtQuick/
│   ├── QtQml/
│   └── QtQuick/VirtualKeyboard/
└── run.sh
```

### 查看日志

```bash
# 查看所有 Qt 相关日志
adb logcat | grep -i qt

# 查看虚拟键盘日志
adb logcat | grep -i virtualkeyboard

# 查看错误信息
adb logcat | grep -E "(ERROR|FATAL)"
```

## 故障排除

### 键盘不显示

1. 检查是否安装了 Qt Virtual Keyboard 模块
2. 确认环境变量设置正确
3. 查看控制台错误信息
4. 检查插件是否正确部署

```bash
# 在设备上检查插件
adb shell "ls -la /data/local/tmp/qt-keyboard-demo/plugins/platforminputcontexts/"
```

### 中文输入不可用

需要安装中文输入法支持：

```bash
sudo apt-get install qml6-module-qtvirtualkeyboard-plugin-pinyin
```

### 在嵌入式设备上运行

确保设备上有正确的 Qt 库和插件：

```bash
# 检查插件是否存在
adb shell "ls /data/local/tmp/qt-keyboard-demo/plugins/platforminputcontexts/"
# 应该看到 libqtvirtualkeyboardplugin.so
```

### QML 模块加载失败

检查 QML 模块路径：

```bash
# 在设备上
export QML2_IMPORT_PATH=/data/local/tmp/qt-keyboard-demo/qml_modules
export QT_DEBUG_PLUGINS=1
qml6 qml/qt-keyboard-demo.qml
```

### Framebuffer 问题

检查 Framebuffer 设备：

```bash
adb shell "ls -la /dev/fb0"
adb shell "cat /sys/class/graphics/fb0/virtual_size"
```

## 与自定义键盘的对比

| 特性 | Qt Virtual Keyboard | 自定义键盘 (FreeVirtualKeyboard) |
|-----|---------------------|----------------------------------|
| 官方支持 | ✅ Qt 官方 | ❌ 第三方 |
| 多语言 | ✅ 30+ 语言 | 需要自己实现 |
| 手写输入 | ✅ 支持 | 需要自己实现 |
| 自动完成 | ✅ 内置 | 需要自己实现 |
| 定制性 | ⚠️ 有限 | ✅ 完全控制 |
| 体积 | ⚠️ 较大 | ✅ 可控 |
| 许可证 | GPL/商业 | 自定义 |
| 部署复杂度 | ⚠️ 需要额外插件 | ✅ 简单 |

## 参考资料

- [Qt Virtual Keyboard 官方文档](https://doc.qt.io/qt-6/qtvirtualkeyboard-index.html)
- [Qt Virtual Keyboard QML Types](https://doc.qt.io/qt-6/qtvirtualkeyboard-qmlmodule.html)
- [输入法提示 (InputMethodHints)](https://doc.qt.io/qt-6/qt.html#InputMethodHint-enum)
- [Qt for Embedded Linux](https://doc.qt.io/qt-6/embedded-linux.html)
