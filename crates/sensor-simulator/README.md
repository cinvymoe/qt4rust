# sensor-simulator

传感器数据模拟器库，用于开发和测试。

## 功能特性

- **正弦波模拟器**: 生成周期性波形数据
- **可配置参数**: 振幅、频率、偏移量、噪声等
- **实时数据生成**: 基于时间的动态数据

## 使用示例

```rust
use sensor_simulator::prelude::*;

// 创建配置
let config = SimulatorConfig {
    amplitude: 5.0,
    frequency: 0.5,
    offset: 15.0,
    noise_level: 0.1,
};

// 创建模拟器
let simulator = SineSimulator::new(config);

// 生成数据
let value = simulator.generate();
println!("Sensor value: {}", value);
```

## 应用场景

- 起重机载荷模拟
- 传感器数据测试
- 开发环境数据源
