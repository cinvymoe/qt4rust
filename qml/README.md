# QML 项目结构说明

## 目录结构

```
qml/
├── main.cpp              # C++ 入口（占位符，实际由 Rust 实现）
├── main.qml              # QML 主入口文件
├── main.preview.qml      # 预览文件
├── components/           # 可复用组件
│   ├── controls/         # 基础控件（Button, Input）
│   ├── layouts/          # 布局组件
│   └── dialogs/          # 弹窗组件
├── views/                # 页面级组件
│   ├── HomeView.qml      # 主页面
│   └── SettingsView.qml  # 设置页面
├── assets/               # 资源文件
│   ├── images/           # 图片资源
│   ├── fonts/            # 字体资源
│   └── translations/     # 翻译文件
└── styles/               # 主题配置
    ├── Theme.qml         # 主题单例
    └── qmldir            # QML 模块定义
```

## 组件说明

- **components/controls**: 基础 UI 控件，如按钮、输入框等
- **components/layouts**: 布局组件，定义页面结构
- **components/dialogs**: 对话框和弹窗组件
- **views**: 完整的页面视图
- **styles**: 主题和样式配置，使用单例模式
- **assets**: 静态资源文件

## 使用方式

在 QML 文件中导入：

```qml
import "views"
import "styles"
import "components/controls"
```

使用主题：

```qml
color: Theme.backgroundColor
font.pixelSize: Theme.fontSizeLarge
```
