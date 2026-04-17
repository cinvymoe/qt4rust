// 多速率管道测试
//
// 测试: 采集(10ms) → 滤波缓冲 → 计算(100ms) → 存储(500ms)
//
// 运行: cargo run --example test_multi_rate_pipeline

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

mod mock_repository {
    use qt_rust_demo::models::SensorData;
    use std::sync::{Arc, Mutex};

    #[allow(dead_code)]
    pub struct MockRepository {
        counter: Arc<Mutex<u64>>,
    }

    #[allow(dead_code)]
    impl MockRepository {
        pub fn new() -> Self {
            Self {
                counter: Arc::new(Mutex::new(0)),
            }
        }

        pub fn get_latest_sensor_data(&self) -> Result<SensorData, String> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            let t = (*counter as f64) * 0.1;
            let load = 15.0 + 5.0 * (t * 0.5).sin();
            let radius = 8.0 + 3.0 * (t * 0.3).cos();
            let angle = 60.0 + 10.0 * (t * 0.2).sin();
            Ok(SensorData::from_tuple(load, radius, angle, false, false))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("=== 多速率管道测试 ===\n");

    // 测试1: FilterBuffer 基本功能
    println!("[TEST 1] FilterBuffer 基本功能测试");
    test_filter_buffer_basic()?;

    // 测试2: 多速率数据流
    println!("\n[TEST 2] 多速率数据流测试");
    test_multi_rate_flow().await?;

    println!("\n=== 所有测试通过 ===");
    Ok(())
}

fn test_filter_buffer_basic() -> Result<(), String> {
    use qt_rust_demo::models::SensorData;
    use qt_rust_demo::pipeline::{FilterBuffer, FilterBufferConfig, FilterType};

    println!("  - 测试无滤波 (None)");
    {
        let config = FilterBufferConfig {
            filter_type: FilterType::None,
            window_size: 5,
        };
        let mut buffer = FilterBuffer::new(config);
        buffer.push(SensorData::from_tuple(10.0, 5.0, 30.0, false, false));
        buffer.push(SensorData::from_tuple(20.0, 6.0, 31.0, false, false));
        let result = buffer.get_filtered().unwrap();
        assert_eq!(result.ad1_load(), 20.0);
        println!("    ✓ None滤波返回最新数据: ad1={}", result.ad1_load());
    }

    println!("  - 测试均值滤波 (Mean)");
    {
        let config = FilterBufferConfig {
            filter_type: FilterType::Mean,
            window_size: 5,
        };
        let mut buffer = FilterBuffer::new(config);
        for i in 0..5 {
            buffer.push(SensorData::from_tuple((i + 1) as f64 * 10.0, 5.0, 30.0, false, false));
        }
        let result = buffer.get_filtered().unwrap();
        assert_eq!(result.ad1_load(), 30.0);
        println!("    ✓ Mean滤波正确: ad1={} (期望30)", result.ad1_load());
    }

    println!("  - 测试中值滤波 (Median)");
    {
        let config = FilterBufferConfig {
            filter_type: FilterType::Median,
            window_size: 5,
        };
        let mut buffer = FilterBuffer::new(config);
        buffer.push(SensorData::from_tuple(10.0, 5.0, 30.0, false, false));
        buffer.push(SensorData::from_tuple(50.0, 5.0, 30.0, false, false));
        buffer.push(SensorData::from_tuple(20.0, 5.0, 30.0, false, false));
        buffer.push(SensorData::from_tuple(40.0, 5.0, 30.0, false, false));
        buffer.push(SensorData::from_tuple(30.0, 5.0, 30.0, false, false));
        let result = buffer.get_filtered().unwrap();
        assert_eq!(result.ad1_load(), 30.0);
        println!("    ✓ Median滤波正确: ad1={} (期望30)", result.ad1_load());
    }

    println!("  - 测试窗口溢出 (窗口3,数据5)");
    {
        let config = FilterBufferConfig {
            filter_type: FilterType::Mean,
            window_size: 3,
        };
        let mut buffer = FilterBuffer::new(config);
        for i in 0..5 {
            buffer.push(SensorData::from_tuple((i + 1) as f64 * 10.0, 5.0, 30.0, false, false));
        }
        let result = buffer.get_filtered().unwrap();
        assert_eq!(result.ad1_load(), 40.0);
        println!(
            "    ✓ 窗口溢出正确: ad1={} (期望40,只用最近3条)",
            result.ad1_load()
        );
    }

    println!("  - 测试is_ready状态");
    {
        let config = FilterBufferConfig {
            filter_type: FilterType::Mean,
            window_size: 3,
        };
        let mut buffer = FilterBuffer::new(config);
        assert!(!buffer.is_ready());
        buffer.push(SensorData::from_tuple(10.0, 5.0, 30.0, false, false));
        assert!(!buffer.is_ready());
        buffer.push(SensorData::from_tuple(20.0, 5.0, 30.0, false, false));
        assert!(!buffer.is_ready());
        buffer.push(SensorData::from_tuple(30.0, 5.0, 30.0, false, false));
        assert!(buffer.is_ready());
        println!("    ✓ is_ready正确: 0/1/2条时false,3条时true");
    }

    Ok(())
}

async fn test_multi_rate_flow() -> Result<(), String> {
    use qt_rust_demo::config::config_manager::ConfigManager;
    use qt_rust_demo::models::crane_config::CraneConfig;
    use qt_rust_demo::pipeline::{FilterBuffer, FilterBufferConfig, FilterType};
    use qt_rust_demo::pipeline::{ProcessPipeline, ProcessPipelineConfig};
    use qt_rust_demo::pipeline::{ProcessedDataBuffer, SharedBuffer};
    use qt_rust_demo::repositories::CraneDataRepository;
    use std::sync::RwLock;

    println!("  - 初始化组件...");

    let collection_interval_ms = 10u64;
    let filter_window_size = 10;
    let process_interval_ms = 100u64;
    let test_duration_ms = 500u64;

    // 创建滤波缓冲区
    let filter_config = FilterBufferConfig {
        filter_type: FilterType::Mean,
        window_size: filter_window_size,
    };
    let filter_buffer: Arc<Mutex<FilterBuffer>> =
        Arc::new(Mutex::new(FilterBuffer::new(filter_config)));

    // 创建显示缓冲区
    let display_buffer: SharedBuffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));

    // 创建配置
    let crane_config = Arc::new(CraneConfig::default());
    let config_manager = Arc::new(ConfigManager::default());
    let repository = Arc::new(CraneDataRepository::new(config_manager));

    // 创建计算管道
    let process_config = ProcessPipelineConfig {
        interval: Duration::from_millis(process_interval_ms),
    };
    let mut process_pipeline = ProcessPipeline::new(
        process_config,
        Arc::clone(&filter_buffer),
        Arc::clone(&display_buffer),
        Arc::clone(&crane_config),
    );

    println!(
        "  - 配置: 采集={}ms, 滤波窗口={}, 计算={}ms, 测试={}ms",
        collection_interval_ms, filter_window_size, process_interval_ms, test_duration_ms
    );

    // 启动计算管道
    process_pipeline.start();
    println!("  - 计算管道已启动");

    // 模拟采集: 10ms间隔
    println!("  - 开始模拟采集...");
    let collection_interval = Duration::from_millis(collection_interval_ms);
    let total_collections = (test_duration_ms / collection_interval_ms) as usize;

    for i in 0..total_collections {
        tokio::time::sleep(collection_interval).await;

        let sensor_data = repository
            .get_latest_sensor_data()
            .map_err(|e| format!("采集失败: {}", e))?;

        filter_buffer.lock().unwrap().push(sensor_data);

        if i % 5 == 0 {
            println!("    采集进度: {}/{}", i, total_collections);
        }
    }

    // 等待计算管道处理
    println!("  - 等待计算管道处理...");
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 检查结果
    let display_data = display_buffer.read().unwrap();
    let latest = display_data.get_latest();

    process_pipeline.stop();

    if let Some(data) = latest {
        println!(
            "  ✓ 处理成功: seq={}, load={:.2}t, radius={:.2}m, angle={:.2}°, moment={:.1}%",
            data.sequence_number,
            data.current_load,
            data.working_radius,
            data.boom_angle,
            data.moment_percentage
        );
    } else {
        return Err("没有处理后的数据".to_string());
    }

    // 验证滤波缓冲区状态
    let fb = filter_buffer.lock().unwrap();
    println!(
        "  ✓ 滤波缓冲区状态: len={}, ready={}",
        fb.len(),
        fb.is_ready()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_buffer_module() {
        test_filter_buffer_basic().unwrap();
    }

    #[tokio::test]
    async fn test_multi_rate_module() {
        test_multi_rate_flow().await.unwrap();
    }
}
