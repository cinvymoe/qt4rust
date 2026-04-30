# 3D 汽车吊可视化组件设计文档

**版本**: 1.0  
**日期**: 2026-04-30  
**状态**: 待用户审批  
**功能**: 在监控页面中使用 3D 汽车吊模型显示臂架状态

---

## 1. 概述

### 1.1 背景

当前起重机监控系统在监控页面 (`MonitoringView.qml`) 中使用静态 PNG 图片 (`canvas-crane.png`) 显示臂架状态。该图片不响应臂架角度和长度变化，仅作为装饰性图形。

### 1.2 需求

将静态 2D 图片替换为可交互的 3D 汽车吊模型，并关联以下实时数据：
- **臂架角度** (`boom_angle`): 0° - 90°
- **臂架长度** (`boom_length`): 伸缩臂长度
- **载荷状态** (`current_load` / `moment_percentage`): 显示安全/预警/报警状态

### 1.3 目标平台

- **目标**: ARM32 嵌入式设备
- **性能约束**: 低多边形模型，优化渲染

---

## 2. 技术方案

### 2.1 技术选型

| 技术点 | 选择 | 理由 |
|--------|------|------|
| 3D 引擎 | Qt Quick 3D | Qt 6 官方推荐，ARM 优化好，与 QML 无缝集成 |
| 模型格式 | glTF 2.0 (.glb) | 二进制格式加载快，支持 PBR 材质 |
| 动态加载 | RuntimeLoader | 运行时加载，可切换模型 |
| 变换方式 | eulerRotation + scale | QML 原生绑定，简单高效 |

### 2.2 3D 模型来源

**推荐模型**:

| 模型 | 链接 | 多边形 | 格式 | 许可 |
|------|------|--------|------|------|
| **Truck Mounted Mobile Crane** (推荐) | [CGTrader #6904659](https://www.cgtrader.com/3d-models/vehicle/industrial-vehicle/truck-mounted-mobile-crane-heavy-machinery-construction-vehicle) | 25,815 | glTF 4.3MB | 免费 (Royalty Free) |
| Mobile Hydraulic Truck Crane | [Sketchfab](https://sketchfab.com/3d-models/mobile-hydraulic-truck-crane-telescopic-8c343b1ce4724631b4623e9ef7778163) | 36,000 | glTF | CC Attribution |

---

## 3. 架构设计

### 3.1 组件层次结构

```
┌─────────────────────────────────────────────────────────────┐
│                      MonitoringView.qml                     │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Crane3DView (新组件)                       ││
│  │  ┌───────────────────────────────────────────────────┐  ││
│  │  │  View3D                                          │  ││
│  │  │    ├── PerspectiveCamera (固定视角)             │  ││
│  │  │    ├── DirectionalLight (单向光照)              │  ││
│  │  │    ├── AmbientLight (环境光)                    │  ││
│  │  │    │                                              │  ││
│  │  │    └── Node (root) ─────────────────────────     │  ││
│  │  │        ├── Chassis (车体 - 固定)                 │  ││
│  │  │        ├── Turret (转台 - 固定)                  │  ││
│  │  │        │     └── BoomPivot (臂架关节)            │  ││
│  │  │        │          ├── BoomArm (臂架)             │  ││
│  │  │        │          │   └── Hook (吊钩)             │  ││
│  │  │        │          │        └── Cable (钢索)       │  ││
│  │  │        │          └── Extension (伸缩段)         │  ││
│  │  │        └── Outrigger (支腿 - 固定)               │  ││
│  │  └───────────────────────────────────────────────────┘  ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
         ↑                              ↑
    boom_angle ──→ BoomPivot         boom_length ──→ Extension
                    .eulerRotation.z          .scale.y
    
    current_load ──→ Hook
                    .material.baseColor (颜色)
                    .material.emissive (发光)
```

### 3.2 文件变更

| 操作 | 文件路径 | 说明 |
|------|----------|------|
| 新增 | `qml/components/controls/Crane3DView.qml` | 3D 可视化组件 |
| 新增 | `qml/assets/models/truck_crane.glb` | 3D 模型文件 |
| 修改 | `qml/views/MonitoringView.qml` | 替换静态 Image 为 Crane3DView |
| 修改 | `crates/qt-app/build.rs` | 添加 Qt Quick 3D 模块 |

### 3.3 数据绑定

| QML 属性 | 数据源 | 用途 |
|----------|--------|------|
| `craneView.boomAngle` | `viewModel.boom_angle` | 控制臂架仰角 (0-90°) |
| `craneView.boomLength` | `viewModel.boom_length` | 控制臂架长度伸缩 |
| `craneView.momentPercentage` | `viewModel.moment_percentage` | 控制吊钩颜色状态 |

---

## 4. 接口设计

### 4.1 Crane3DView 组件属性

```qml
// Crane3DView.qml 公开接口
Item {
    // 输入属性 - 由父组件绑定
    property real boomAngle: 0          // 臂架角度 (0-90°)
    property real boomLength: 22.6      // 臂架长度 (米)
    property real momentPercentage: 0   // 力矩百分比 (0-100%)
    
    // 配置属性
    property real minBoomLength: 10.0   // 最小臂长
    property real maxBoomLength: 35.0   // 最大臂长
    
    // 颜色配置
    property color safeColor: "#22c55e"      // 安全 - 绿色
    property color warningColor: "#f59e0b"   // 预警 - 黄色
    property color dangerColor: "#ef4444"    // 报警 - 红色
    
    // 尺寸
    width: 400
    height: 300
}
```

### 4.2 颜色阈值逻辑

```javascript
function getHookColor(percentage) {
    if (percentage >= 90) {
        return dangerColor;    // 报警 - 红色
    } else if (percentage >= 70) {
        return warningColor;   // 预警 - 黄色
    } else {
        return safeColor;      // 安全 - 绿色
    }
}
```

---

## 5. 性能优化策略 (ARM32)

### 5.1 模型优化

| 优化项 | 策略 |
|--------|------|
| 多边形数 | < 30,000 面 |
| 纹理分辨率 | ≤ 1024x1024 |
| 材质 | PrincipledMaterial，关闭环境光遮蔽 |

### 5.2 渲染优化

| 优化项 | 策略 |
|--------|------|
| 光照 | 单向光 + 环境光，跳过阴影计算 |
| 抗锯齿 | 关闭 multisample |
| 更新频率 | 绑定到现有 500ms Timer，避免高频更新 |
| 渲染模式 | Underlay 模式（全屏时） |

### 5.3 动画策略

```qml
// 使用 NumberAnimation 而非 Timer 更新
NumberAnimation {
    target: boomPivot
    property: "eulerRotation.z"
    to: -root.boomAngle  // 负号：Qt3D 中角度方向
    duration: 400
    easing.type: Easing.InOutCubic
}
```

---

## 6. 依赖变更

### 6.1 系统依赖 (Ubuntu/Debian)

```bash
sudo apt install qt6-quick3d-dev qt6-quick3d-assetutils-dev
```

### 6.2 build.rs 修改

```rust
// crates/qt-app/build.rs
.qt_module("Quick3D")      // 添加
.qt_module("Quick3DAssetImport")  // 添加
```

---

## 7. 验收标准

### 7.1 功能验收

| 测试项 | 预期结果 |
|--------|----------|
| 3D 模型加载 | 应用启动后 3D 模型正确显示 |
| 臂架角度响应 | 拖动滑块或传感器数据变化时，臂架实时旋转 |
| 臂架长度响应 | 臂长变化时，臂架模型相应伸缩 |
| 载荷颜色 | 力矩 ≥90% 显示红色，70-90% 显示黄色，<70% 显示绿色 |

### 7.2 性能验收

| 测试项 | 预期结果 |
|--------|----------|
| 启动时间 | < 3 秒 |
| 帧率 | ≥ 30 FPS |
| 内存占用 | < 150MB |

---

## 8. 后续工作

1. 获取 3D 模型文件 (.glb)
2. 验证模型关节结构与代码绑定的对应关系
3. 测试不同传感器数据下的显示效果
4. 如需多角度观察，添加相机控制

---

**文档状态**: ✅ 设计完成，待用户审批后进入实现阶段