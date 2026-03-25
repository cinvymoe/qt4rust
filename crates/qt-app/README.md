# Qt App - Crane Monitoring GUI Application

这是起重机监控系统的 Qt GUI 应用程序。

## 架构说明

- **lib (qt-rust-demo)**: 核心业务逻辑，不依赖 Qt
- **qt-app**: Qt GUI 应用，依赖 qt-rust-demo lib

这种分离的好处：
1. lib 可以独立编译和测试，不需要 Qt 环境
2. lib 可以被其他项目复用（如 CLI 工具、Web 服务）
3. qt-app 只包含 UI 相关代码，职责清晰

## 编译

### 编译 lib（无 Qt）
```bash
cargo build --lib
```

### 编译 Qt 应用
```bash
cargo build -p qt-app
```

### 交叉编译到 ARM32
```bash
cargo build -p qt-app --target armv7-unknown-linux-gnueabihf --release
```

## 运行

```bash
cargo run -p qt-app
```

## 目录结构

```
crates/qt-app/
├── Cargo.toml          # Qt 应用配置
├── build.rs            # cxx-qt 构建脚本
├── qml/                # QML 文件（符号链接到 ../../qml）
└── src/
    ├── main.rs                         # 应用入口
    ├── application.rs                  # Qt 应用初始化
    ├── monitoring_viewmodel.rs         # 监控 ViewModel
    ├── data_collection_controller.rs   # 数据采集控制器
    └── viewmodel_manager.rs            # ViewModel 管理器
```

## 依赖关系

```
qt-app
  ├── qt-rust-demo (lib)
  │   ├── states
  │   ├── intents
  │   ├── reducers
  │   ├── models
  │   ├── repositories
  │   └── pipeline
  ├── qt-threading-utils
  ├── cxx-qt
  └── cxx-qt-lib
```
