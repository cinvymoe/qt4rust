# Settings ViewModel 模块

这个模块提供了起重机监控系统的设置界面 ViewModel，使用 cxx-qt 实现 Rust 与 QML 的交互。

## 模块结构

```
settings_viewmodel/
├── mod.rs                          # 模块入口
├── settings_main.rs                # 主设置 ViewModel
├── README.md                       # 本文档
└── calibration/                    # 校准设置子模块
    ├── mod.rs                      # 校准模块入口
    ├── alarm_threshold.rs          # 报警阈值设置
    ├── angle_calibration.rs        # 角度传感器校准
    ├── load_calibration.rs         # 载荷传感器校准
    └── radius_calibration.rs       # 半径传感器校准
```

## 功能说明

### 1. 主设置 ViewModel (settings_main.rs)

提供设置界面的主要导航和全局操作。

### 2. 报警阈值设置 (alarm_threshold.rs)

管理力矩和角度的报警阈值，支持保存和重置操作。

### 3. 传感器校准模块

使用两点校准法校准各类传感器：
- 角度传感器 (angle_calibration.rs)
- 载荷传感器 (load_calibration.rs)
- 半径传感器 (radius_calibration.rs)

## 设计特点

1. **模块化设计**: 每个设置功能独立为一个 ViewModel
2. **配置持久化**: 自动加载和保存到 TOML 配置文件
3. **实时更新**: 支持捕获当前传感器读数进行校准
4. **错误处理**: 完善的日志记录和错误处理机制
5. **QML 集成**: 通过 cxx-qt 无缝集成到 Qt/QML 界面

## 依赖关系

所有 ViewModel 依赖主项目 `qt-rust-demo` 的以下模块：
- `config::calibration_manager`: 配置文件管理
- `models::sensor_calibration`: 传感器校准数据模型
