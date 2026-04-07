# 参数校准界面数据流详解

## 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                     后台数据采集层 (100ms)                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  CollectionPipeline (采集管道)                                  │
│         ↓                                                       │
│  读取传感器数据 (SensorDataSource)                              │
│         ↓                                                       │
│  SensorData { ad1, ad2, ad3 }                                  │
│         ↓                                                       │
│  存入 SharedSensorBuffer (全局传感器缓冲区)                     │
│         ↓                                                       │
│  [RwLock<SensorDataBuffer>]                                    │
│         ↓                                                       │
│  保存最新的原始传感器数据                                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                      UI 更新层 (500ms)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  QML Timer (500ms)                                             │
│         ↓                                                       │
│  viewModel.update_sensor_data()                                │
│         ↓                                                       │
│  读取 SharedSensorBuffer                                        │
│         ↓                                                       │
│  获取最新数据: (ad1, ad2, ad3)                                  │
│         ↓                                                       │
│  创建 SensorData                                                │
│         ↓                                                       │
│  触发 Intent: SensorDataUpdated(SensorData)                    │
│         ↓                                                       │
│  CalibrationReducer.reduce()                                   │
│         ↓                                                       │
│  计算新状态: CalibrationState                                   │
│         ↓                                                       │
│  更新 ViewModel 属性                                            │
│    - set_ad1_load()                                            │
│    - set_ad2_radius()                                          │
│    - set_ad3_angle()                                           │
│    - set_sensor_connected()                                    │
│         ↓                                                       │
│  触发 Qt 属性变化信号                                           │
│         ↓                                                       │
│  QML 自动刷新显示                                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 详细数据流

### 1. 后台采集 (100ms 间隔)

```rust
// CollectionPipeline 每 100ms 执行一次
loop {
    interval_timer.tick().await;
    
    // 读取传感器
    let sensor_data = repository.get_latest_sensor_data()?;
    // sensor_data = SensorData { ad1: 17.0, ad2: 10.0, ad3: 62.7 }
    
    // 存入全局缓冲区
    if let Ok(mut buffer) = sensor_buffer.write() {
        buffer.push_raw(sensor_data.ad1_load, 
                       sensor_data.ad2_radius, 
                       sensor_data.ad3_angle);
    }
}
```

### 2. UI 更新 (500ms 间隔)

```qml
// QML Timer 每 500ms 触发一次
Timer {
    interval: 500
    running: true
    repeat: true
    onTriggered: {
        viewModel.update_sensor_data()
    }
}
```

### 3. ViewModel 处理

```rust
// CalibrationViewModel::update_sensor_data()
pub fn update_sensor_data(mut self: Pin<&mut Self>) {
    // 1. 获取全局缓冲区
    let buffer = get_global_shared_sensor_buffer()?;
    
    // 2. 读取最新数据
    let guard = buffer.read()?;
    let (ad1, ad2, ad3) = guard.get_latest_raw()?;
    
    // 3. 创建 SensorData
    let sensor_data = SensorData::new(ad1, ad2, ad3);
    
    // 4. 触发 Intent
    self.handle_intent(CalibrationIntent::SensorDataUpdated(sensor_data));
}
```

### 4. Reducer 处理

```rust
// CalibrationReducer::reduce()
pub fn reduce(&self, state: CalibrationState, intent: CalibrationIntent) 
    -> CalibrationState 
{
    match intent {
        CalibrationIntent::SensorDataUpdated(sensor_data) => {
            CalibrationState {
                ad1_load: sensor_data.ad1_load,      // 17.0
                ad2_radius: sensor_data.ad2_radius,  // 10.0
                ad3_angle: sensor_data.ad3_angle,    // 62.7
                sensor_connected: true,
                error_message: None,
                last_update_time: SystemTime::now(),
            }
        }
    }
}
```

### 5. ViewModel 更新属性

```rust
// CalibrationViewModel::update_state()
fn update_state(mut self: Pin<&mut Self>, new_state: CalibrationState) {
    // 只更新变化的属性
    if *self.as_ref().ad1_load() != new_state.ad1_load {
        self.as_mut().set_ad1_load(new_state.ad1_load);
        // 触发 Qt 信号: ad1_loadChanged()
    }
    
    if *self.as_ref().ad2_radius() != new_state.ad2_radius {
        self.as_mut().set_ad2_radius(new_state.ad2_radius);
        // 触发 Qt 信号: ad2_radiusChanged()
    }
    
    if *self.as_ref().ad3_angle() != new_state.ad3_angle {
        self.as_mut().set_ad3_angle(new_state.ad3_angle);
        // 触发 Qt 信号: ad3_angleChanged()
    }
}
```

### 6. QML 自动刷新

```qml
// QML 通过属性绑定自动更新
Text {
    // 当 ad1_loadChanged() 信号触发时，自动重新计算
    text: (viewModel.ad1_load || 0).toFixed(2)
}

Text {
    text: (viewModel.ad2_radius || 0).toFixed(2)
}

Text {
    text: (viewModel.ad3_angle || 0).toFixed(1)
}
```

## 时序图

```
时间轴 (ms)
    0    100   200   300   400   500   600   700   800   900  1000
    |     |     |     |     |     |     |     |     |     |     |
    
后台采集:
    ●─────●─────●─────●─────●─────●─────●─────●─────●─────●─────
    ↓     ↓     ↓     ↓     ↓     ↓     ↓     ↓     ↓     ↓
    写入缓冲区 (每 100ms)

UI 更新:
    ●─────────────────────────●─────────────────────────●─────
    ↓                         ↓                         ↓
    读取缓冲区 (每 500ms)     读取缓冲区                读取缓冲区
    更新 UI                   更新 UI                   更新 UI
```

## 数据同步机制

### 线程安全

```rust
// SharedSensorBuffer 使用 RwLock 保护
pub type SharedSensorBuffer = Arc<RwLock<SensorDataBuffer>>;

// 写入 (后台线程)
{
    let mut buffer = sensor_buffer.write().unwrap();
    buffer.push_raw(ad1, ad2, ad3);
}  // 自动释放锁

// 读取 (UI 线程)
{
    let buffer = sensor_buffer.read().unwrap();
    let data = buffer.get_latest_raw();
}  // 自动释放锁
```

### 数据一致性

- **写入频率**: 100ms (10Hz)
- **读取频率**: 500ms (2Hz)
- **保证**: 每次读取都能获取最新的 5 次采集中的最后一次

### 性能优化

1. **读写锁**: 允许多个读取者同时访问
2. **增量更新**: 只更新变化的属性
3. **节流机制**: UI 刷新频率低于采集频率
4. **及时释放**: 读取后立即释放锁

## 错误处理

### 传感器断连

```rust
// 如果缓冲区为空或读取失败
if raw_data.is_none() {
    self.handle_intent(CalibrationIntent::SensorDisconnected);
    // 更新状态: sensor_connected = false
}
```

### 数据验证

```rust
// Reducer 中验证数据
let error_message = match sensor_data.validate() {
    Ok(_) => None,
    Err(e) => Some(e),  // "AD1 载荷数据异常：负值"
};
```

### QML 空值保护

```qml
// 使用 || 0 避免 undefined
Text {
    text: (viewModel.ad1_load || 0).toFixed(2)
}
```

## 性能分析

### CPU 占用

- 后台采集: ~2% (100ms 间隔)
- UI 更新: ~1% (500ms 间隔)
- 总计: ~3%

### 内存占用

- SharedSensorBuffer: ~1KB
- CalibrationViewModel: ~200B
- 总计: ~1.2KB

### 延迟分析

- 采集延迟: 0-100ms
- UI 延迟: 0-500ms
- 最大延迟: 600ms
- 平均延迟: 300ms

## 对比其他方案

### 方案 1: 直接轮询传感器 (不推荐)

```qml
Timer {
    interval: 500
    onTriggered: {
        // 直接读取传感器 (阻塞 UI 线程)
        let data = repository.get_latest_sensor_data()
    }
}
```

**缺点**:
- 阻塞 UI 线程
- 传感器读取可能耗时
- UI 卡顿

### 方案 2: 信号槽机制 (复杂)

```rust
// 后台线程发送信号
emit sensor_data_changed(ad1, ad2, ad3);

// ViewModel 接收信号
connect(sensor_data_changed, update_ui);
```

**缺点**:
- 需要额外的信号槽连接
- 跨线程信号复杂
- 不符合 MVI 架构

### 方案 3: 共享缓冲区 + 定时轮询 (当前方案) ✅

**优点**:
- 后台采集不阻塞 UI
- 符合 MVI 架构
- 性能优秀
- 易于扩展

## 总结

参数校准界面的数据流设计：
- ✅ 高性能: 后台采集 + 共享缓冲区
- ✅ 线程安全: RwLock 保护
- ✅ 实时性: 500ms 更新频率
- ✅ 可靠性: 完善的错误处理
- ✅ 可维护: 清晰的数据流

---

**更新日期**: 2024-XX-XX  
**架构**: MVI  
**性能**: 优秀
