use modbus_tcp::ModbusDataSource;
use sensor_core::source::{CombinedSensorSource, SimulatedAnalogSource, SimulatedDigitalInput};
use sensor_core::{init_builtin_sources, AnalogSource, SensorSource};
use std::path::Path;
use std::time::Duration;

fn main() {
    println!("=== Sensor Core 数据采集测试 (Modbus TCP) ===\n");

    init_builtin_sources();

    let args: Vec<String> = std::env::args().collect();
    let use_modbus = args.iter().any(|a| a == "--modbus" || a == "-m");
    let config_path = "config/modbus_sensors.toml";

    if use_modbus && Path::new(config_path).exists() {
        println!("[INFO] 使用 Modbus TCP 模式");
        println!("[INFO] 配置文件: {}\n", config_path);
        test_modbus_source(config_path);
    } else if use_modbus {
        println!("[WARN] 配置文件不存在: {}", config_path);
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
            Ok((ad1, ad2, ad3, di0, di1)) => {
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

fn test_modbus_source(config_path: &str) {
    println!("--- Modbus TCP 数据源测试 ---\n");

    let config_content = match std::fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            println!("[ERROR] 读取配置失败: {}", e);
            return;
        }
    };

    let mut modbus_source = match ModbusDataSource::from_config(&config_content) {
        Ok(source) => source,
        Err(e) => {
            println!("[ERROR] 解析配置失败: {:?}", e);
            return;
        }
    };

    println!("[INFO] 连接 Modbus TCP 服务器...");
    if let Err(e) = modbus_source.connect() {
        println!("[ERROR] 连接失败: {:?}", e);
        println!("[INFO] 请确保 Modbus TCP 服务器正在运行");
        return;
    }

    println!("[INFO] 连接成功!\n");
    println!("连接状态: {}", modbus_source.is_connected());
    println!();

    let digital = Box::new(SimulatedDigitalInput::new(10));
    let combined = CombinedSensorSource::new(
        Box::new(ModbusAnalogWrapper {
            inner: modbus_source,
        }),
        digital,
    );

    println!("采集数据 (Ctrl+C 退出):");
    loop {
        match combined.read_all() {
            Ok((ad1, ad2, ad3, di0, di1)) => {
                let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
                println!(
                    "  [{}] AD1={:.0}, AD2={:.0}, AD3={:.0}, DI0={}, DI1={}",
                    timestamp, ad1, ad2, ad3, di0, di1
                );
            }
            Err(e) => {
                println!("  [ERROR] 读取失败: {}", e);
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

struct ModbusAnalogWrapper {
    inner: ModbusDataSource,
}

impl AnalogSource for ModbusAnalogWrapper {
    fn read(&self) -> sensor_traits::SensorResult<(f64, f64, f64)> {
        self.inner
            .read_all()
            .map_err(|e| sensor_traits::SensorError::ReadError(format!("Modbus 读取失败: {:?}", e)))
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    fn source_name(&self) -> &str {
        "ModbusAnalogSource"
    }
}
