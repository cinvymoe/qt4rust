use sensor_core::source::{CombinedSensorSource, SimulatedAnalogSource, SimulatedDigitalInput};
use sensor_core::{init_builtin_sources, AnalogSource, SensorSource};
use sensor_traits::SensorReading;
use std::time::Duration;

fn main() {
    println!("=== Sensor Core 数据采集测试 ===\n");

    init_builtin_sources();

    let args: Vec<String> = std::env::args().collect();
    let use_modbus = args.iter().any(|a| a == "--modbus" || a == "-m");

    if use_modbus {
        println!("[INFO] Modbus TCP 模式需要配置，请使用模拟器模式");
        println!("[INFO] 回退到模拟器模式\n");
        test_simulator_source();
    } else {
        println!("[INFO] 使用模拟器模式 (添加 --modbus 参数启用 Modbus TCP)\n");
        test_simulator_source();
    }

    println!("\n=== 测试完成 ===");
}

fn test_simulator_source() {
    println!("--- 模拟器数据源测试 ---\n");

    let analog = Box::new(SimulatedAnalogSource::new());
    let digital = Box::new(SimulatedDigitalInput::new(10));
    let combined = CombinedSensorSource::new(analog, digital);

    println!("连接状态: {}", combined.is_connected());
    println!();

    println!("采集 10 次数据:");
    for i in 1..=10 {
        match combined.read_all() {
            Ok(reading) => {
                let (ad1, ad2, ad3, di0, di1) = reading.to_tuple();
                println!(
                    "  [{}] AD1={:.0}, AD2={:.0}, AD3={:.0}, DI0={}, DI1={}",
                    i, ad1, ad2, ad3, di0, di1
                );
            }
            Err(e) => {
                println!("  [{}] 读取失败: {}", i, e);
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}
