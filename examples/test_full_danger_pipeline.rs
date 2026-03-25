// 完整的采集管道危险状态切换测试
// 模拟真实的采集流程：采集 → 处理 → 检测危险 → 保存报警

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use qt_rust_demo::models::{SensorData, ProcessedData};
use qt_rust_demo::repositories::mock_storage_repository::MockStorageRepository;
use qt_rust_demo::repositories::storage_repository::StorageRepository;
use qt_rust_demo::pipeline::storage_pipeline::{StoragePipeline, StoragePipelineConfig};
use qt_rust_demo::pipeline::shared_buffer::ProcessedDataBuffer;
use std::sync::RwLock;

/// 模拟采集管道的数据处理逻辑
async fn simulate_collection_cycle(
    pipeline: &StoragePipeline,
    sensor_data: SensorData,
    sequence: u64,
    last_was_danger: &mut bool,
) -> ProcessedData {
    // 1. 采集传感器数据
    let processed = ProcessedData::from_sensor_data(sensor_data, sequence);
    
    println!("  [采集] seq={}, load={:.1}t, moment={:.1}%, is_danger={}", 
             processed.sequence_number,
             processed.current_load,
             processed.moment_percentage,
             processed.is_danger);
    
    // 2. 检测危险状态切换
    let current_danger = processed.is_danger;
    
    if current_danger {
        // 当前是危险状态
        if *last_was_danger {
            // 上次也是危险 → 持续报警，跳过
            println!("  [报警] 持续报警，跳过保存");
        } else {
            // 上次不是危险 → 新报警，保存
            println!("  [报警] 新报警！保存报警记录");
            pipeline.save_alarm_async(processed.clone());
            *last_was_danger = true;
        }
    } else {
        // 当前不是危险状态
        if *last_was_danger {
            // 上次是危险 → 报警解除
            println!("  [报警] 报警解除，通知 storage_pipeline");
            pipeline.notify_danger_cleared();
            *last_was_danger = false;
        } else {
            // 上次也不是危险 → 正常状态
            println!("  [正常] 无报警");
        }
    }
    
    processed
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== 完整采集管道危险状态切换测试 ===\n");
    
    // 创建存储管道
    let repo = Arc::new(MockStorageRepository::new());
    let buffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));
    let config = StoragePipelineConfig::default();
    let pipeline = StoragePipeline::new(
        config,
        repo.clone() as Arc<dyn StorageRepository>,
        buffer,
    ).await.expect("Failed to create storage pipeline");
    
    println!("✓ 存储管道创建成功\n");
    
    // 模拟采集管道的状态
    let mut last_was_danger = false;
    let mut sequence = 0u64;
    
    // 模拟采集循环
    println!("【模拟采集循环】\n");
    
    // 周期 1-3: 正常状态
    println!("周期 1-3: 正常载荷");
    for _i in 1..=3 {
        sequence += 1;
        let sensor_data = SensorData::new(10.0, 10.0, 60.0); // 低载荷
        simulate_collection_cycle(&pipeline, sensor_data, sequence, &mut last_was_danger).await;
        sleep(Duration::from_millis(50)).await;
    }
    println!();
    
    // 周期 4: 首次报警
    println!("周期 4: 载荷升高，触发报警");
    sequence += 1;
    let sensor_data = SensorData::new(23.0, 10.0, 60.0); // 高载荷
    simulate_collection_cycle(&pipeline, sensor_data, sequence, &mut last_was_danger).await;
    sleep(Duration::from_millis(100)).await;
    println!();
    
    // 周期 5-7: 持续报警
    println!("周期 5-7: 载荷持续高位");
    for i in 5..=7 {
        sequence += 1;
        let sensor_data = SensorData::new(24.0 + (i as f64 * 0.1), 10.0, 60.0);
        simulate_collection_cycle(&pipeline, sensor_data, sequence, &mut last_was_danger).await;
        sleep(Duration::from_millis(50)).await;
    }
    println!();
    
    // 周期 8: 报警解除
    println!("周期 8: 载荷下降，报警解除");
    sequence += 1;
    let sensor_data = SensorData::new(12.0, 10.0, 60.0); // 载荷下降
    simulate_collection_cycle(&pipeline, sensor_data, sequence, &mut last_was_danger).await;
    sleep(Duration::from_millis(100)).await;
    println!();
    
    // 周期 9-10: 正常状态
    println!("周期 9-10: 正常载荷");
    for _ in 9..=10 {
        sequence += 1;
        let sensor_data = SensorData::new(8.0, 10.0, 60.0);
        simulate_collection_cycle(&pipeline, sensor_data, sequence, &mut last_was_danger).await;
        sleep(Duration::from_millis(50)).await;
    }
    println!();
    
    // 周期 11: 再次报警
    println!("周期 11: 载荷再次升高");
    sequence += 1;
    let sensor_data = SensorData::new(25.0, 10.0, 60.0);
    simulate_collection_cycle(&pipeline, sensor_data, sequence, &mut last_was_danger).await;
    sleep(Duration::from_millis(100)).await;
    println!();
    
    // 周期 12: 报警解除
    println!("周期 12: 载荷下降");
    sequence += 1;
    let sensor_data = SensorData::new(10.0, 10.0, 60.0);
    simulate_collection_cycle(&pipeline, sensor_data, sequence, &mut last_was_danger).await;
    sleep(Duration::from_millis(100)).await;
    println!();
    
    // 验证结果
    let alarm_count = repo.get_alarm_count();
    println!("=== 测试结果 ===");
    println!("总采集周期: {}", sequence);
    println!("报警记录数: {}", alarm_count);
    println!();
    
    // 预期：2 次报警（周期 4 和周期 11）
    assert_eq!(alarm_count, 2, "应该保存 2 条报警记录");
    
    println!("✓ 测试通过！");
    println!("  - 首次报警（周期 4）: 已保存");
    println!("  - 持续报警（周期 5-7）: 已跳过");
    println!("  - 报警解除（周期 8）: 状态已重置");
    println!("  - 再次报警（周期 11）: 已保存");
    println!("  - 报警解除（周期 12）: 状态已重置");
    
    // 等待异步任务完成
    sleep(Duration::from_millis(200)).await;
}
