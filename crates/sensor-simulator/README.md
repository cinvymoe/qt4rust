# sensor-simulator

统一的传感器库，支持模拟传感器和真实传感器接口。

## 功能特性

- **统一接口**: 通过 `SensorProvider` trait 提供统一的传感器访问接口
- **模拟传感器**: 支持正弦波、随机数等多种模拟器
- **真实传感器**: 支持串口、网络等真实传感器接口
- **可配置参数**: 灵活的配置系统
- **错误处理**: 完善的错误类型定义

## 使用示例

### 使用模拟传感器

```rust
use sensor_simulator::prelude::*;

// 创建模拟传感器配置
let config = SimulatorConfig {
    amplitude: 5.0,
    frequency: 0.5,
    offset: 15.0,
    noise_level: 0.1,
};

// 创建模拟传感器
let sensor = SimulatedSensor::new(SimulatorType::Sine(config));

// 读取数据
match sensor.read() {
    Ok(value) => println!("Sensor value: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}
```

### 使用真实传感器

```rust
use sensor_simulator::prelude::*;

// 创建串口传感器配置
let config = SerialConfig {
    port: "/dev/ttyUSB0".to_string(),
    baud_rate: 9600,
    timeout_ms: 1000,
};

// 创建真实传感器
let sensor = RealSensor::new(SensorType::Serial(config));

// 读取数据
match sensor.read() {
    Ok(value) => println!("Sensor value: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}
```

### 使用统一接口

```rust
use sensor_simulator::prelude::*;

fn process_sensor_data<S: SensorProvider>(sensor: &S) -> Result<f64, SensorError> {
    sensor.read()
}

// 可以传入任何实现了 SensorProvider 的类型
let simulated = SimulatedSensor::default();
let value = process_sensor_data(&simulated)?;
```

## 应用场景

- 起重机载荷监测（开发/生产环境切换）
- 传感器数据测试和验证
- 硬件抽象层实现
