# 虚拟键盘 Demo 总结

本项目包含三个不同的虚拟键盘演示实现。

## Demo 对比

| 特性 | FreeVirtualKeyboard | Qt5 Virtual Keyboard | Qt6 Virtual Keyboard |
|------|---------------------|----------------------|----------------------|
| QML 文件 | `keyboard-demo.qml` | `qt5-keyboard-demo.qml` | `qt-keyboard-demo.qml` |
| Qt 版本 | Qt 5.15+ | Qt 5.15+ | Qt 6.4+ |
| 依赖模块 | FreeVirtualKeyboard 1.0 | QtQuick.VirtualKeyboard 2.15 | QtQuick.VirtualKeyboard |
| 设备兼容性 | ✅ 已测试可用 | ⚠️ 需要安装模块 | ❌ 设备不支持 Qt6 |
| 部署复杂度 | 简单 | 中等 | 复杂 |
| 推荐使用 | ✅ 推荐 | 可选 | 不推荐（设备限制） |

## 1. FreeVirtualKeyboard Demo（推荐）

### 文件
- `qml/keyboard-demo.qml`

### 特点
- 使用项目自带的虚拟键盘实现
- 无需额外安装模块
- 已在设备上测试通过
- 支持中英文输入

### 运行方式

#### 在设备上运行
```bash
./scripts/deploy-keyboard.sh
./scripts/run-on-device.sh
```

#### 本地预览
```bash
python3 scripts/preview_qml.py
```

### 输入类型
- 普通文本
- 密码输入
- 大写/小写字母
- 电话号码
- 纯数字
- 多行文本
- 中文输入

## 2. Qt5 Virtual Keyboard Demo

### 文件
- `qml/qt5-keyboard-demo.qml`

### 特点
- 使用 Qt 官方 Virtual Keyboard 模块
- 需要在设备上安装 `qml-module-qtquick-virtualkeyboard`
- 支持更多语言和输入法
- 功能更丰富（手写、预测输入等）

### 前置条件

设备上需要安装：
```bash
# 在设备上执行
apt-get install qml-module-qtquick-virtualkeyboard
```

### 运行方式

#### 部署到设备
```bash
./scripts/deploy-qt5-keyboard-simple.sh
adb shell "sh /data/local/tmp/qt-keyboard-demo/run.sh"
```

#### 本地预览
```bash
python3 scripts/preview_qt_keyboard.py
```

### 输入类型
- 普通文本
- 密码输入
- 邮箱地址
- 数字输入
- 电话号码
- URL
- 多行文本

## 3. Qt6 Virtual Keyboard Demo

### 文件
- `qml/qt-keyboard-demo.qml`

### 特点
- 使用最新的 Qt6 Virtual Keyboard
- 语法更现代（无版本号导入）
- 功能最完整

### 限制
- 当前设备运行 Qt 5.15，不支持 Qt6
- 仅适用于开发环境预览

### 运行方式

#### 本地预览（需要 Qt6）
```bash
# 安装 PySide6
pip install PySide6

# 运行预览
python3 scripts/preview_qt_keyboard.py
```

## 推荐使用场景

### 场景 1: 在 ADB 设备上运行
**推荐**: FreeVirtualKeyboard Demo

```bash
./scripts/deploy-keyboard.sh
./scripts/run-on-device.sh
```

**原因**:
- 无需额外安装
- 已测试可用
- 部署简单

### 场景 2: 开发环境测试
**推荐**: Qt5 或 Qt6 Virtual Keyboard Demo

```bash
python3 scripts/preview_qt_keyboard.py
```

**原因**:
- 功能更丰富
- 官方支持
- 开发体验好

### 场景 3: 需要多语言支持
**推荐**: Qt Virtual Keyboard（需要在设备上安装）

**原因**:
- 支持 30+ 种语言
- 内置手写识别
- 智能预测输入

## 快速开始

### 最简单的方式（推荐新手）

```bash
# 1. 部署到设备
./scripts/deploy-keyboard.sh

# 2. 运行
./scripts/run-on-device.sh
```

### 如果想使用 Qt Virtual Keyboard

```bash
# 1. 在设备上安装模块
adb shell "apt-get install qml-module-qtquick-virtualkeyboard"

# 2. 部署
./scripts/deploy-qt5-keyboard-simple.sh

# 3. 运行
adb shell "sh /data/local/tmp/qt-keyboard-demo/run.sh"
```

## 故障排除

### 问题: module "QtQuick.VirtualKeyboard" is not installed

**解决方案**: 使用 FreeVirtualKeyboard Demo 或在设备上安装模块

```bash
./scripts/deploy-keyboard.sh
```

### 问题: wrong cpu architecture

**解决方案**: 不要推送 x86_64 的库到 ARM 设备，使用设备上已有的库

### 问题: 键盘不显示

**检查**:
1. 确认使用了正确的 demo 文件
2. 检查设备上是否安装了所需模块
3. 查看日志: `adb logcat | grep -i keyboard`

## 相关文档

- `QT_KEYBOARD_DEMO.md` - Qt Virtual Keyboard 详细文档
- `QT_KEYBOARD_DEVICE_GUIDE.md` - 设备部署指南
- `KEYBOARD_DEMOS_SUMMARY.md` - 本文档

## 总结

对于大多数用户，推荐使用 **FreeVirtualKeyboard Demo**，因为它：
- ✅ 开箱即用
- ✅ 无需额外安装
- ✅ 已在设备上测试
- ✅ 部署简单

如果需要更高级的功能（多语言、手写等），可以考虑在设备上安装 Qt Virtual Keyboard 模块后使用官方实现。
