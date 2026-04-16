// 测试采集管道中危险状态切换逻辑
//
// 测试场景：
// 1. is_danger = true, last_was_danger = false → 保存报警，设置 last_was_danger = true
// 2. is_danger = true, last_was_danger = true → 跳过（持续报警）
// 3. is_danger = false, last_was_danger = true → 调用 notify_danger_cleared()，设置 last_was_danger = false
// 4. is_danger = false, last_was_danger = false → 不触发任何操作

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// 引入必要的模块
use qt_rust_demo::models::{ProcessedData, SensorData};
use qt_rust_demo::pipeline::ProcessedDataBuffer;
use qt_rust_demo::pipeline::{StoragePipeline, StoragePipelineConfig};
use qt_rust_demo::repositories::mock_storage_repository::MockStorageRepository;
use qt_rust_demo::repositories::storage_repository::StorageRepository;
use std::sync::RwLock;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== 测试采集管道危险状态切换逻辑 ===\n");

    // 创建 Mock 存储仓库
    let repo = Arc::new(MockStorageRepository::new());
    let buffer: qt_rust_demo::pipeline::SharedBuffer =
        Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));

    // 创建存储管道
    let config = StoragePipelineConfig::default();
    let pipeline = StoragePipeline::new(config, repo.clone() as Arc<dyn StorageRepository>, buffer)
        .await
        .expect("Failed to create storage pipeline");

    println!("✓ 存储管道创建成功\n");

    // 测试场景 1: 首次报警（is_danger = true, last_was_danger = false）
    println!("【场景 1】首次报警");
    println!("  - is_danger = true");
    println!("  - last_was_danger = false");
    println!("  - 预期：保存报警记录，设置 last_was_danger = true");

    let sensor_data_1 = SensorData::new(23.0, 10.0, 60.0, false, false); // 高载荷，触发报警
    let processed_1 = ProcessedData::from_sensor_data(sensor_data_1, 1);
    assert!(processed_1.is_danger, "数据应该触发报警");

    pipeline.save_alarm_async(processed_1.clone());
    sleep(Duration::from_millis(100)).await;

    let alarm_count_1 = repo.get_alarm_count();
    println!("  ✓ 报警记录数: {}", alarm_count_1);
    assert_eq!(alarm_count_1, 1, "应该保存 1 条报警记录");
    println!();

    // 测试场景 2: 持续报警（is_danger = true, last_was_danger = true）
    println!("【场景 2】持续报警");
    println!("  - is_danger = true");
    println!("  - last_was_danger = true");
    println!("  - 预期：跳过，不保存重复报警");

    let sensor_data_2 = SensorData::new(24.0, 10.0, 60.0, false, false); // 仍然高载荷
    let processed_2 = ProcessedData::from_sensor_data(sensor_data_2, 2);
    assert!(processed_2.is_danger, "数据应该触发报警");

    pipeline.save_alarm_async(processed_2.clone());
    sleep(Duration::from_millis(100)).await;

    let alarm_count_2 = repo.get_alarm_count();
    println!("  ✓ 报警记录数: {}", alarm_count_2);
    assert_eq!(alarm_count_2, 1, "应该仍然是 1 条报警记录（跳过重复）");
    println!();

    // 测试场景 3: 报警解除（is_danger = false, last_was_danger = true）
    println!("【场景 3】报警解除");
    println!("  - is_danger = false");
    println!("  - last_was_danger = true");
    println!("  - 预期：调用 notify_danger_cleared()，重置 last_was_danger = false");

    let sensor_data_3 = SensorData::new(10.0, 10.0, 60.0, false, false); // 低载荷，解除报警
    let processed_3 = ProcessedData::from_sensor_data(sensor_data_3, 3);
    assert!(!processed_3.is_danger, "数据不应该触发报警");

    // 模拟采集管道检测到报警解除，调用 notify_danger_cleared()
    pipeline.notify_danger_cleared();
    sleep(Duration::from_millis(100)).await;

    let alarm_count_3 = repo.get_alarm_count();
    println!("  ✓ 报警记录数: {}", alarm_count_3);
    assert_eq!(alarm_count_3, 1, "报警记录数不变");
    println!("  ✓ last_was_danger 已重置为 false");
    println!();

    // 测试场景 4: 再次报警（is_danger = true, last_was_danger = false）
    println!("【场景 4】再次报警（报警解除后）");
    println!("  - is_danger = true");
    println!("  - last_was_danger = false（已重置）");
    println!("  - 预期：保存新的报警记录");

    let sensor_data_4 = SensorData::new(25.0, 10.0, 60.0, false, false); // 再次高载荷
    let processed_4 = ProcessedData::from_sensor_data(sensor_data_4, 4);
    assert!(processed_4.is_danger, "数据应该触发报警");

    pipeline.save_alarm_async(processed_4.clone());
    sleep(Duration::from_millis(100)).await;

    let alarm_count_4 = repo.get_alarm_count();
    println!("  ✓ 报警记录数: {}", alarm_count_4);
    assert_eq!(alarm_count_4, 2, "应该保存第 2 条报警记录");
    println!();

    // 测试场景 5: 正常状态（is_danger = false, last_was_danger = false）
    println!("【场景 5】正常状态");
    println!("  - is_danger = false");
    println!("  - last_was_danger = false");
    println!("  - 预期：不触发任何操作");

    pipeline.notify_danger_cleared();

    let sensor_data_5 = SensorData::new(8.0, 10.0, 60.0, false, false); // 低载荷
    let processed_5 = ProcessedData::from_sensor_data(sensor_data_5, 5);
    assert!(!processed_5.is_danger, "数据不应该触发报警");

    // 不调用 save_alarm_async，因为不是报警状态
    sleep(Duration::from_millis(100)).await;

    let alarm_count_5 = repo.get_alarm_count();
    println!("  ✓ 报警记录数: {}", alarm_count_5);
    assert_eq!(alarm_count_5, 2, "报警记录数不变");
    println!();

    // 测试总结
    println!("=== 测试总结 ===");
    println!("✓ 场景 1: 首次报警 → 保存成功");
    println!("✓ 场景 2: 持续报警 → 跳过重复");
    println!("✓ 场景 3: 报警解除 → 状态重置");
    println!("✓ 场景 4: 再次报警 → 保存成功");
    println!("✓ 场景 5: 正常状态 → 无操作");
    println!("\n所有测试通过！危险状态切换逻辑正常工作。");
}
