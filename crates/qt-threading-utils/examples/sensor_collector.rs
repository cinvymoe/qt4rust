// 传感器数据采集示例
// 使用 qt-threading-utils 和 sensor-simulator 读取模拟传感器数据 (ad1, ad2, ad3)

use qt_threading_utils::prelude::*;
use sensor_simulator::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 传感器数据结构
#[derive(Debug, Clone, Default)]
pub struct SensorData {
    pub ad1: f64,
    pub ad2: f64,
    pub ad3: f64,
    pub timestamp: u64,
}

/// 多传感器采集器
pub struct MultiSensorCollector {
    /// AD1 传感器 - 随机模拟 (范围: 0-100)
    ad1_sensor: SimulatedSensor,
    /// AD2 传感器 - 正弦波模拟 (振幅: 10, 频率: 0.5Hz, 偏移: 50)
    ad2_sensor: SimulatedSensor,
    /// AD3 传感器 - 随机模拟 (范围: 20-80)
    ad3_sensor: SimulatedSensor,
    /// 最新数据
    latest_data: Arc<Mutex<SensorData>>,
}

impl MultiSensorCollector {
    pub fn new() -> Self {
        Self {
            // AD1: 随机传感器，范围 0-100
            ad1_sensor: SimulatedSensor::new(SimulatorType::Random {
                min: 0.0,
                max: 100.0,
            }),
            // AD2: 正弦波传感器，模拟周期性变化
            ad2_sensor: SimulatedSensor::new(SimulatorType::Sine(SimulatorConfig {
                amplitude: 10.0,
                frequency: 0.5,
                offset: 50.0,
                noise_level: 0.5,
            })),
            // AD3: 随机传感器，范围 20-80
            ad3_sensor: SimulatedSensor::new(SimulatorType::Random {
                min: 20.0,
                max: 80.0,
            }),
            latest_data: Arc::new(Mutex::new(SensorData::default())),
        }
    }

    /// 读取所有传感器数据
    pub fn read_all(&self) -> SensorData {
        let ad1 = self.ad1_sensor.read().unwrap_or(0.0);
        let ad2 = self.ad2_sensor.read().unwrap_or(0.0);
        let ad3 = self.ad3_sensor.read().unwrap_or(0.0);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        SensorData {
            ad1,
            ad2,
            ad3,
            timestamp,
        }
    }

    /// 获取最新数据
    pub fn get_latest(&self) -> SensorData {
        self.latest_data.lock().unwrap().clone()
    }

    /// 更新最新数据
    fn update_latest(&self, data: SensorData) {
        let mut latest = self.latest_data.lock().unwrap();
        *latest = data;
    }

    /// 获取数据共享引用
    pub fn data_handle(&self) -> Arc<Mutex<SensorData>> {
        Arc::clone(&self.latest_data)
    }
}

impl Default for MultiSensorCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() {
    println!("=== 传感器数据采集示例 ===\n");
    println!("使用 qt-threading-utils 定期读取模拟传感器数据");
    println!("传感器: AD1 (随机 0-100), AD2 (正弦波), AD3 (随机 20-80)\n");

    // 创建多传感器采集器
    let collector = Arc::new(MultiSensorCollector::new());

    // 数据计数器
    let counter = Arc::new(Mutex::new(0u32));

    // 创建数据采集定时器 (每 500ms 采集一次)
    println!("1. 启动周期性定时器采集 (500ms 间隔)");
    println!("-------------------------------------------");
    println!(
        "{:<6} {:<12} {:<12} {:<12} {:<16}",
        "序号", "AD1", "AD2", "AD3", "时间戳"
    );
    println!("-------------------------------------------");

    let collector_clone = Arc::clone(&collector);
    let counter_clone = Arc::clone(&counter);

    let timer = PeriodicTimer::new(Duration::from_millis(500));
    timer
        .start(move || {
            let data = collector_clone.read_all();
            let mut count = counter_clone.lock().unwrap();
            *count += 1;

            // 更新最新数据
            collector_clone.update_latest(data.clone());

            println!(
                "{:<6} {:<12.2} {:<12.2} {:<12.2} {:<16}",
                *count, data.ad1, data.ad2, data.ad3, data.timestamp
            );
        })
        .await;

    // 运行 5 秒
    tokio::time::sleep(Duration::from_secs(5)).await;

    // 停止定时器
    timer.stop().await;
    println!("-------------------------------------------");
    println!("   定时器已停止\n");

    // 使用 DataCollector 采集数据
    println!("\n2. 使用 DataCollector 采集数据 (300ms 间隔)");
    println!("-------------------------------------------");
    println!(
        "{:<6} {:<12} {:<12} {:<12} {:<16}",
        "序号", "AD1", "AD2", "AD3", "时间戳"
    );
    println!("-------------------------------------------");

    // 重置计数器
    {
        let mut c = counter.lock().unwrap();
        *c = 0;
    }

    let collector_clone2 = Arc::clone(&collector);
    let counter_clone2 = Arc::clone(&counter);

    let data_collector = DataCollector::new(Duration::from_millis(300));

    data_collector
        .start(move || {
            let data = collector_clone2.read_all();
            let mut count = counter_clone2.lock().unwrap();
            *count += 1;

            println!(
                "{:<6} {:<12.2} {:<12.2} {:<12.2} {:<16}",
                *count, data.ad1, data.ad2, data.ad3, data.timestamp
            );
        })
        .await;

    // 运行 3 秒
    tokio::time::sleep(Duration::from_secs(3)).await;

    // 停止采集器
    data_collector.stop().await;
    println!("-------------------------------------------");
    println!("   采集器已停止\n");

    // 最终状态
    let final_data = collector.get_latest();
    println!("\n=== 最终传感器数据状态 ===");
    println!("AD1: {:.2}", final_data.ad1);
    println!("AD2: {:.2}", final_data.ad2);
    println!("AD3: {:.2}", final_data.ad3);
    println!("时间戳: {}", final_data.timestamp);

    println!("\n=== 示例完成 ===");
}
