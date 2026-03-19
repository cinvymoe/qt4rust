# 数据采集与 ViewModel 集成文档

## 概述

本文档说明了如何在 QML 中绑定 MonitoringViewModel，以及后台数据采集器如何与 ViewModel 连接，实现实时数据更新和 UI 刷新。

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    QML View Layer                           │
│              (MonitoringView.qml)                           │
│                                                             │
│  MonitoringViewModel (Qt Object)                           │
│    ↓ registerToManager()                                   │
├─────────────────────────────────────────────────────────────┤
│              ViewModelManager (Rust)                        │
│         (全局单例，管理数据采集)                              │
│                                                             │
│  DataCollectionController (QML Singleton)                  │
│    ↓ startCollection()                                     │
├─────────────────────────────────────────────────────────────┤
│              DataCollector (Rust)                           │
│         (后台线程，100ms 采集间隔)                            │
│                  ↓                                          │
├─────────────────────────────────────────────────────────────┤
│           CraneDataRepository (Rust)                        │
│                  ↓                                          │
├─────────────────────────────────────────────────────────────┤
│          SensorDataSource (Rust)                            │
│         (模拟传感器数据生成)                                  │
└─────────────────────────────────────────────────────────────┘
```

## 实现步骤

### 1. QML 中绑定 MonitoringViewModel

**文件**: `qml/views/MonitoringView.qml`

```qml
import QtQuick
import qt.rust.demo  // 导入 Rust ViewModel

Item {
    id: monitoringView
    
    // 绑定 ViewModel
    MonitoringViewModel {
        id: viewModel
        
        // 组件创建完成后注册到管理器
        Component.onCompleted: {
            console.log("[QML] MonitoringViewModel created, registering...")
            viewModel.registerToManager()
            
            // 延迟启动数据采集
            Qt.callLater(function() {
                console.log("[QML] Starting data collection...")
                DataCollectionController.startCollection()
            })
        }
        
        Component.onDestruction: {
            console.log("[QML] MonitoringViewModel destroyed, stopping...")
            DataCollectionController.stopCollection()
        }
    }
    
    // 数据绑定示例
    Text {
        text: "当前载荷: " + viewModel.currentLoad.toFixed(1) + " 吨"
    }
    
    Text {
        text: "力矩百分比: " + viewModel.momentPercentage.toFixed(1) + "%"
    }
    
    // 危险状态指示
    Rectangle {
        visible: viewModel.isDanger
        color: "red"
    }
    
    // 传感器连接状态
    Rectangle {
        visible: !viewModel.sensorConnected
        color: "orange"
        Text {
            text: "传感器连接断开"
        }
    }
    
    // 错误提示
    InfoDialog {
        visible: viewModel.errorMessage !== ""
        message: viewModel.errorMessage
        onAccepted: viewModel.clearError()
    }
}
```

### 2. ViewModel 属性说明

**文件**: `src/monitoring_viewmodel.rs`

MonitoringViewModel 暴露以下 Qt 属性给 QML：

| 属性名 | 类型 | 说明 |
|--------|------|------|
| `currentLoad` | f64 | 当前载荷（吨） |
| `ratedLoad` | f64 | 额定载荷（吨） |
| `workingRadius` | f64 | 工作半径（米） |
| `boomAngle` | f64 | 吊臂角度（度） |
| `boomLength` | f64 | 臂长（米） |
| `momentPercentage` | f64 | 力矩百分比 |
| `isDanger` | bool | 是否处于危险状态 |
| `sensorConnected` | bool | 传感器连接状态 |
| `errorMessage` | QString | 错误信息 |

### 3. ViewModel 方法说明

| 方法名 | 说明 |
|--------|------|
| `clearError()` | 清除错误信息 |
| `resetAlarm()` | 重置报警状态 |
| `registerToManager()` | 注册到全局管理器（自动调用） |

### 4. 数据采集流程

```
1. QML 创建 MonitoringViewModel
   ↓
2. Component.onCompleted 调用 registerToManager()
   ↓
3. ViewModelManager 标记 ViewModel 已准备好
   ↓
4. QML 调用 DataCollectionController.startCollection()
   ↓
5. ViewModelManager 启动 DataCollector
   ↓
6. DataCollector 在后台线程每 100ms 采集一次数据
   ↓
7. 采集到的数据通过 Intent 传递
   ↓
8. Reducer 计算新状态
   ↓
9. ViewModel 更新 Qt 属性
   ↓
10. QML 通过属性绑定自动刷新 UI
```

### 5. 数据更新机制

#### 5.1 后台数据采集

**文件**: `src/collector/data_collector.rs`

```rust
// 100ms 采集间隔
let handle = thread::spawn(move || {
    while running.load(Ordering::Relaxed) {
        // 从 Repository 读取数据
        match repository.get_latest_sensor_data() {
            Ok(sensor_data) => {
                // 触发回调，传递 Intent
                on_data(MonitoringIntent::SensorDataUpdated(sensor_data));
            }
            Err(e) => {
                on_data(MonitoringIntent::SensorDisconnected);
            }
        }
        
        // 100ms 间隔
        thread::sleep(Duration::from_millis(100));
    }
});
```

#### 5.2 状态转换

**文件**: `src/reducers/monitoring_reducer.rs`

```rust
pub fn reduce(&self, state: MonitoringState, intent: MonitoringIntent) -> MonitoringState {
    match intent {
        MonitoringIntent::SensorDataUpdated(sensor_data) => {
            // 计算力矩百分比
            let moment_percentage = sensor_data.calculate_moment_percentage();
            
            // 判断是否危险
            let is_danger = moment_percentage >= 90.0;
            
            // 返回新状态
            MonitoringState {
                current_load: sensor_data.ad1_load,
                working_radius: sensor_data.ad2_radius,
                boom_angle: sensor_data.ad3_angle,
                moment_percentage,
                is_danger,
                sensor_connected: true,
                ..state
            }
        }
        // ... 其他 Intent 处理
    }
}
```

#### 5.3 ViewModel 更新

**文件**: `src/monitoring_viewmodel.rs`

```rust
pub fn handle_intent(mut self: Pin<&mut Self>, intent: MonitoringIntent) {
    // 1. 从 Qt 属性重建当前状态
    let current_state = MonitoringState { /* ... */ };
    
    // 2. 调用 Reducer 计算新状态
    let new_state = self.reducer.reduce(current_state, intent);
    
    // 3. 更新状态（只更新变化的属性）
    self.update_state(new_state);
}

fn update_state(mut self: Pin<&mut Self>, new_state: MonitoringState) {
    // 只更新变化的属性，避免不必要的 UI 刷新
    if *self.as_ref().current_load() != new_state.current_load {
        self.as_mut().set_current_load(new_state.current_load);
    }
    // ... 其他属性
}
```

## 测试方法

### 测试 1: 验证 ViewModel 绑定

1. 编译项目：
   ```bash
   cargo build --target armv7-unknown-linux-gnueabihf
   ```

2. 运行应用：
   ```bash
   ./target/armv7-unknown-linux-gnueabihf/debug/qt-rust-demo
   ```

3. 观察控制台输出：
   ```
   [INFO] ViewModelManager initialized
   [QML] MonitoringViewModel created, registering...
   [INFO] ViewModel marked as ready
   [QML] Starting data collection...
   [INFO] Starting data collection from QML...
   [INFO] Data collection started
   ```

### 测试 2: 验证数据更新

1. 观察 UI 中的数据卡片是否显示数据
2. 数据应该每 100ms 更新一次
3. 观察控制台输出：
   ```
   [DATA] Collected: SensorDataUpdated(SensorData { ad1_load: 17.5, ... })
   [DATA] Collected: SensorDataUpdated(SensorData { ad1_load: 18.2, ... })
   ```

### 测试 3: 验证危险状态

1. 等待力矩百分比超过 90%
2. 观察 UI 中的危险卡片是否显示
3. 观察顶部栏的报警指示是否激活

### 测试 4: 验证错误处理

1. 模拟传感器断连（修改代码返回错误）
2. 观察 UI 中的传感器断连提示是否显示
3. 观察错误对话框是否弹出

## 当前限制和未来改进

### 当前限制

1. **线程安全问题**: 
   - 当前实现中，后台线程直接打印日志，未实际更新 ViewModel
   - 需要实现线程安全的通信机制

2. **数据采集与 ViewModel 未完全连接**:
   - `DataCollector` 的回调中只打印日志
   - 需要通过 Qt 信号槽机制将数据传递给 ViewModel

3. **模拟数据**:
   - 当前使用随机数模拟传感器数据
   - 未来需要接入真实的串口/CAN总线数据

### 未来改进方案

#### 方案 1: 使用 Qt 信号槽（推荐）

在 ViewModel 中定义信号：

```rust
#[cxx_qt::bridge]
pub mod monitoring_viewmodel_bridge {
    extern "RustQt" {
        #[qobject]
        type MonitoringViewModel = super::MonitoringViewModelRust;
        
        // 定义信号
        #[qsignal]
        fn sensor_data_received(self: Pin<&mut MonitoringViewModel>, data: SensorData);
    }
}

// 后台线程发送信号
vm.sensor_data_received(sensor_data);

// ViewModel 连接信号到槽
impl MonitoringViewModel {
    fn setup_connections(mut self: Pin<&mut Self>) {
        self.as_mut().on_sensor_data_received(|vm, data| {
            vm.handle_intent(MonitoringIntent::SensorDataUpdated(data));
        });
    }
}
```

#### 方案 2: 使用 QTimer 在主线程轮询

在 QML 中使用 Timer 定期调用 ViewModel 方法：

```qml
Timer {
    interval: 100
    running: true
    repeat: true
    onTriggered: {
        viewModel.refreshData()
    }
}
```

#### 方案 3: 使用 Channel 和 QMetaObject::invokeMethod

使用 Rust 的 channel 传递数据，然后在主线程处理：

```rust
let (tx, rx) = mpsc::channel();

// 后台线程发送数据
tx.send(sensor_data).unwrap();

// 主线程接收数据
if let Ok(data) = rx.try_recv() {
    vm.handle_intent(MonitoringIntent::SensorDataUpdated(data));
}
```

## 调试技巧

### 1. 启用详细日志

在代码中添加更多 `eprintln!` 语句：

```rust
eprintln!("[DEBUG] State changed: {:?} -> {:?}", old_state, new_state);
eprintln!("[INTENT] Handling: {:?}", intent);
eprintln!("[DATA] Sensor data: load={}, radius={}", data.ad1_load, data.ad2_radius);
```

### 2. QML 调试

在 QML 中使用 `console.log`：

```qml
onCurrentLoadChanged: {
    console.log("Load changed:", currentLoad)
}
```

### 3. 检查属性绑定

在 QML 中添加调试文本：

```qml
Text {
    text: "Debug: load=" + viewModel.currentLoad + 
          ", moment=" + viewModel.momentPercentage +
          ", danger=" + viewModel.isDanger
}
```

## 总结

本文档说明了：

1. ✅ 在 QML 中绑定 MonitoringViewModel
2. ✅ 实现后台数据采集器的基础架构
3. ✅ 创建 ViewModel 管理器和数据采集控制器
4. ⚠️ 数据采集与 ViewModel 的连接（部分实现）
5. ✅ 测试数据更新和 UI 刷新的方法

当前实现已经完成了基础架构，但数据采集与 ViewModel 的实际连接还需要进一步完善（使用信号槽或其他线程安全机制）。

---

**文档更新日期**: 2026-03-18  
**实施状态**: ✅ 基础架构完成，⚠️ 线程通信待完善
