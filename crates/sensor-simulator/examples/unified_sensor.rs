// 统一传感器接口使用示例

use sensor_simulator::prelude::*;

fn main() {
    println!("=== 统一传感器接口示例 ===\n");

    // 示例 1: 使用模拟传感器（正弦波）
    println!("1. 正弦波模拟传感器:");
    let sine_config = SimulatorConfig {
        amplitude: 10.0,
        frequency: 0.2,
        offset: 50.0,
        noise_level: 0.5,
    };
    let sine_sensor = SimulatedSensor::new(SimulatorType::Sine(sine_config));
    demo_sensor(&sine_sensor);

    // 示例 2: 使用随机数模拟传感器
    println!("\n2. 随机数模拟传感器:");
    let random_sensor = SimulatedSensor::new(SimulatorType::Random {
        min: 0.0,
        max: 100.0,
    });
    demo_sensor(&random_sensor);

    // 示例 3: 使用常量模拟传感器
    println!("\n3. 常量模拟传感器:");
    let constant_sensor = SimulatedSensor::new(SimulatorType::Constant(42.0));
    demo_sensor(&constant_sensor);

    // 示例 4: 真实传感器（串口）
    println!("\n4. 真实串口传感器:");
    let serial_config = SerialConfig {
        port: "/dev/ttyUSB0".to_string(),
        baud_rate: 9600,
        timeout_ms: 1000,
    };
    let mut serial_sensor = RealSensor::new(SensorType::Serial(serial_config));
    
    // 连接传感器
    match serial_sensor.connect() {
        Ok(_) => println!("传感器连接成功"),
        Err(e) => println!("传感器连接失败: {}", e),
    }
    
    demo_sensor(&serial_sensor);

    // 示例 5: 向后兼容的旧 API
    println!("\n5. 旧版 SineSimulator API (向后兼容):");
    let legacy_simulator = SineSimulator::default();
    println!("传感器名称: {}", legacy_simulator.name());
    for i in 0..3 {
        let value = legacy_simulator.generate();
        println!("  读取 {}: {:.2}", i + 1, value);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// 演示统一的传感器接口
fn demo_sensor<S: SensorProvider>(sensor: &S) {
    println!("传感器名称: {}", sensor.name());
    println!("连接状态: {}", if sensor.is_connected() { "已连接" } else { "未连接" });
    
    // 读取多次数据
    for i in 0..3 {
        match sensor.read() {
            Ok(value) => println!("  读取 {}: {:.2}", i + 1, value),
            Err(e) => println!("  读取 {} 失败: {}", i + 1, e),
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
