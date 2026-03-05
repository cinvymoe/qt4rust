# Qt Virtual Keyboard 在 ADB 设备上的部署指南

## 问题总结

在尝试部署 Qt Virtual Keyboard Demo 到 ADB 设备时遇到以下问题：

1. **设备运行 Qt 5.15**，而不是 Qt 6
2. **设备上未安装 Qt Virtual Keyboard 模块**
3. **架构不匹配**：不能推送 x86_64 的库到 ARM 设备

## 当前设备状态

```bash
# 设备上的 Qt 版本
$ adb shell "qml --version"
Qml Runtime 5.15.11

# 设备上的 Qt 库
$ adb shell "ls /usr/lib/libQt5*.so.5"
/usr/lib/libQt5Core.so.5
/usr/lib/libQt5Gui.so.5
/usr/lib/libQt5Network.so.5
...

# Virtual Keyboard 模块：未安装
$ adb shell "find /usr -name '*VirtualKeyboard*'"
(无结果)
```

## 解决方案

### 方案 1: 在设备上安装 Qt Virtual Keyboard（推荐）

如果设备支持包管理器（如 apt），可以直接在设备上安装：

```bash
# 连接到设备
adb shell

# 安装 Qt5 Virtual Keyboard
apt-get update
apt-get install qml-module-qtquick-virtualkeyboard

# 或者如果是 Qt6
apt-get install qml6-module-qtquick-virtualkeyboard
```

安装后重新运行：

```bash
adb shell "sh /data/local/tmp/qt-keyboard-demo/run.sh"
```

### 方案 2: 使用已有的 FreeVirtualKeyboard

项目中已经有一个自定义的虚拟键盘实现，可以直接使用：

```bash
# 使用现有的键盘 demo
./scripts/deploy-keyboard.sh
./scripts/run-on-device.sh
```

这个版本使用的是 `qml/keyboard-demo.qml`，它导入的是 `FreeVirtualKeyboard 1.0`，这是项目自带的实现。

### 方案 3: 交叉编译 Qt Virtual Keyboard

如果设备不支持包管理器，需要交叉编译 Qt Virtual Keyboard 模块：

#### 步骤 1: 准备交叉编译环境

```bash
# 安装交叉编译工具链
sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf

# 下载 Qt Virtual Keyboard 源码
git clone https://code.qt.io/qt/qtvirtualkeyboard.git
cd qtvirtualkeyboard
git checkout v5.15.11  # 匹配设备上的 Qt 版本
```

#### 步骤 2: 配置交叉编译

```bash
mkdir build-arm && cd build-arm

# 配置 qmake 进行交叉编译
/path/to/arm/qt5/bin/qmake .. \
    -spec linux-arm-gnueabihf-g++ \
    CONFIG+=release
```

#### 步骤 3: 编译和部署

```bash
make -j$(nproc)

# 推送到设备
adb push plugins/platforminputcontexts/libqtvirtualkeyboardplugin.so \
    /data/local/tmp/qt-keyboard-demo/plugins/platforminputcontexts/

adb push qml/QtQuick/VirtualKeyboard \
    /data/local/tmp/qt-keyboard-demo/qml_modules/QtQuick/
```

### 方案 4: 创建无键盘版本（测试用）

创建一个不依赖虚拟键盘的简化版本，用于测试基本功能：

```qml
// qml/simple-input-demo.qml
import QtQuick 2.15
import QtQuick.Controls 2.15

ApplicationWindow {
    visible: true
    width: 1280
    height: 800
    title: "Simple Input Demo"

    Column {
        anchors.centerIn: parent
        spacing: 20

        Text {
            text: "使用物理键盘或触摸输入"
            font.pixelSize: 24
        }

        TextField {
            width: 400
            placeholderText: "输入文本..."
            font.pixelSize: 18
        }
    }
}
```

## 推荐做法

根据你的需求选择：

1. **如果设备有包管理器** → 使用方案 1（在设备上安装）
2. **如果只是测试功能** → 使用方案 2（使用 FreeVirtualKeyboard）
3. **如果需要官方 Virtual Keyboard** → 使用方案 3（交叉编译）
4. **如果只需要基本输入** → 使用方案 4（无键盘版本）

## 验证安装

安装完成后，验证模块是否可用：

```bash
# 检查插件
adb shell "ls -la /usr/lib/qt5/plugins/platforminputcontexts/*virtual*"

# 检查 QML 模块
adb shell "ls -la /usr/lib/qt5/qml/QtQuick/VirtualKeyboard/"

# 测试导入
adb shell "qml -c 'import QtQuick.VirtualKeyboard 2.15'"
```

## 常见错误

### 错误 1: module "QtQuick.VirtualKeyboard" is not installed

**原因**: 设备上没有安装 Virtual Keyboard 模块

**解决**: 使用上述方案 1、2 或 3

### 错误 2: wrong cpu architecture

**原因**: 推送了错误架构的库文件

**解决**: 确保使用 ARM 架构的库，不要使用 x86_64 的库

### 错误 3: Library import requires a version

**原因**: QML 导入语句缺少版本号

**解决**: 使用 `import QtQuick.VirtualKeyboard 2.15` 而不是 `import QtQuick.VirtualKeyboard`

## 文件清单

项目中创建的相关文件：

```
qml/
├── qt-keyboard-demo.qml          # Qt6 版本（需要 Qt 6.x）
├── qt5-keyboard-demo.qml         # Qt5 版本（需要 Qt 5.15 + Virtual Keyboard）
└── keyboard-demo.qml             # FreeVirtualKeyboard 版本（自带实现）

scripts/
├── deploy-qt-keyboard-demo.sh    # 完整部署脚本
├── deploy-qt5-keyboard-simple.sh # 简化部署脚本
├── run-qt-keyboard-on-device.sh  # 运行脚本
└── install-qt6-virtualkeyboard.sh # 安装脚本（主机端）
```

## 下一步

建议使用项目自带的 FreeVirtualKeyboard 实现：

```bash
# 查看现有的键盘 demo
cat qml/keyboard-demo.qml

# 部署并运行
./scripts/deploy-keyboard.sh
```

这个版本已经过测试，可以在设备上正常运行。
