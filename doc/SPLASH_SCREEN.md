# 启动画面实现说明

## 概述

应用启动时会先显示一个启动画面（Splash Screen），展示应用 Logo、标题和加载进度条，2.5 秒后自动淡出并显示主界面。

## 实现方式

### 1. QML 层实现

启动画面直接集成在 `qml/main.qml` 中，作为一个覆盖层（z-index: 1000）显示在主内容之上。

**关键特性：**
- 深色背景（`Theme.darkBackground`）
- Logo 图片淡入动画（800ms）
- 应用标题和副标题淡入动画
- 进度条加载动画（2000ms）
- 版本信息显示
- 整体淡出动画（500ms）

### 2. 时序控制

```
启动 → 显示启动画面 → 2.5秒后开始淡出 → 0.5秒淡出动画 → 隐藏启动画面 → 显示主界面
```

**定时器配置：**
- `splashTimer`: 2500ms - 触发启动画面淡出
- `hideSplashTimer`: 500ms - 等待淡出动画完成后隐藏

### 3. 动画效果

**Logo 和文本淡入：**
```qml
NumberAnimation on opacity {
    from: 0
    to: 1
    duration: 800
    easing.type: Easing.InOutQuad
}
```

**进度条加载：**
```qml
NumberAnimation on width {
    from: 0
    to: 200
    duration: 2000
    easing.type: Easing.InOutQuad
}
```

**整体淡出：**
```qml
Behavior on opacity {
    NumberAnimation {
        duration: 500
        easing.type: Easing.InOutQuad
    }
}
```

## 设计规范

### 颜色
- 背景色：`Theme.darkBackground` (#0f172b)
- 主标题：`Theme.textPrimary` (#ffffff)
- 副标题：`Theme.textSecondary` (#cad5e2)
- 版本信息：`Theme.textTertiary` (#90a1b9)
- 进度条背景：`Theme.darkSurface` (#1d293d)
- 进度条前景：`Theme.darkAccent` (#2b7fff)

### 字体
- 主标题：`Theme.fontSizeXXLarge` (30px)
- 副标题：`Theme.fontSizeMedium` (16px)
- 版本信息：`Theme.fontSizeSmall` (14px)

### 间距
- 元素间距：`Theme.spacingXLarge` (32px)
- 底部边距：`Theme.spacingLarge` (24px)

### 尺寸
- Logo 尺寸：200x200px
- 进度条：200x4px

## 自定义配置

### 修改显示时长

在 `qml/main.qml` 中修改定时器间隔：

```qml
// 启动画面定时器
Timer {
    id: splashTimer
    interval: 2500  // 修改这里（毫秒）
    running: true
    repeat: false
    onTriggered: {
        splashScreen.opacity = 0
        hideSplashTimer.start()
    }
}
```

### 修改动画时长

修改各个动画的 `duration` 属性：

```qml
// Logo 淡入动画
NumberAnimation on opacity {
    from: 0
    to: 1
    duration: 800  // 修改这里（毫秒）
    easing.type: Easing.InOutQuad
}

// 进度条动画
NumberAnimation on width {
    from: 0
    to: 200
    duration: 2000  // 修改这里（毫秒）
    easing.type: Easing.InOutQuad
}
```

### 更换 Logo

替换 `qml/assets/images/icon-logo.png` 文件，或修改图片路径：

```qml
Image {
    id: logoImage
    source: "assets/images/your-logo.png"  // 修改这里
    width: 200
    height: 200
    anchors.horizontalCenter: parent.horizontalCenter
    fillMode: Image.PreserveAspectFit
}
```

### 修改文本内容

```qml
// 应用标题
Text {
    text: "起重机力矩监测系统"  // 修改这里
    font.pixelSize: Theme.fontSizeXXLarge
    font.family: Theme.fontFamilyDefault
    color: Theme.textPrimary
    anchors.horizontalCenter: parent.horizontalCenter
}

// 副标题
Text {
    text: "Crane Monitoring System"  // 修改这里
    font.pixelSize: Theme.fontSizeMedium
    font.family: Theme.fontFamilyDefault
    color: Theme.textSecondary
    anchors.horizontalCenter: parent.horizontalCenter
}

// 版本信息
Text {
    text: "v1.0.0"  // 修改这里
    font.pixelSize: Theme.fontSizeSmall
    color: Theme.textTertiary
    anchors.bottom: parent.bottom
    anchors.horizontalCenter: parent.horizontalCenter
    anchors.bottomMargin: Theme.spacingLarge
}
```

## 构建和测试

### 编译项目

```bash
make build
```

### 运行应用

```bash
make run
```

### 部署到设备

```bash
make deploy
```

## 技术细节

### 为什么不使用独立窗口？

最初考虑使用独立的启动窗口，但 QML 中不能在 Item 内部创建 Window。因此采用覆盖层方案：
- 启动画面作为高 z-index 的 Rectangle 覆盖在主内容上
- 通过 `visible` 和 `opacity` 属性控制显示/隐藏
- 使用 `Behavior` 实现平滑的淡入淡出效果

### 性能优化

- 启动画面在淡出后会被隐藏（`showSplash = false`），释放资源
- 主界面在启动画面显示期间已经加载完成，切换无延迟
- 所有动画使用硬件加速的 `NumberAnimation`

## 故障排除

### 启动画面不显示

1. 检查 Logo 图片路径是否正确
2. 确认 `Theme.qml` 已正确加载
3. 查看控制台是否有 QML 错误

### 启动画面显示时间过长

1. 检查 `splashTimer.interval` 设置
2. 确认定时器正常触发（添加 `console.log` 调试）

### 动画不流畅

1. 确认使用了 `Easing.InOutQuad` 缓动函数
2. 检查动画 `duration` 是否合理（建议 250-1000ms）
3. 确认硬件支持 OpenGL 加速

## 相关文件

- `qml/main.qml` - 主界面和启动画面
- `qml/styles/Theme.qml` - 设计令牌定义
- `qml/assets/images/icon-logo.png` - Logo 图片
- `src/application.rs` - Rust 应用入口
- `src/main.rs` - 主函数
