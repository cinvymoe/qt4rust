// 完整管道系统演示（纯后台版本，无 Qt 依赖）
//
// 演示如何使用三后台管道架构：
// - 后台线程 1：采集管道（从传感器采集数据）
// - 后台线程 2：存储管道（批量存储运行数据 + 异步存储报警）
// - 主线程：显示管道（待实现）

use std::sync::Arc;
use std::time::Duration;

// 模拟 CraneDataRepository（简化版本）
mod demo_repository {
    use std::sync::{Arc, Mutex};

    pub struct DemoRepository {
        counter: Arc<Mutex<u64>>,
    }

    impl DemoRepository {
        pub fn new() -> Self {
            Self {
                counter: Arc::new(Mutex::new(0)),
            }
        }

        pub fn get_data(&self) -> (f64, f64, f64) {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;

            // 模拟传感器数据
            let t = (*counter as f64) * 0.1;
            let load = 15.0 + 5.0 * (t * 0.5).sin();
            let radius = 8.0 + 3.0 * (t * 0.3).cos();
            let angle = 60.0 + 10.0 * (t * 0.2).sin();

            (load, radius, angle)
        }
    }
}

use demo_repository::DemoRepository;

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("=== 完整管道系统演示（纯后台版本）===\n");

    // 1. 加载配置
    use qt_rust_demo::config::pipeline_config::PipelineConfig;

    println!("[INFO] 加载配置文件...");
    let config = PipelineConfig::load();

    println!("[INFO] 配置参数:");
    println!("  - 采集间隔: {}ms", config.collection.interval_ms);
    println!("  - 存储间隔: {}ms", config.storage.interval_ms);
    println!("  - 批量大小: {}", config.storage.batch_size);
    println!("  - 缓冲区大小: {}", config.collection.buffer_size);
    println!("  - 使用模拟器: {}", config.collection.use_simulator);
    println!("  - 数据库路径: {}\n", config.database.path);

    // 2. 创建数据仓库（简化版本）
    let repository = Arc::new(DemoRepository::new());

    // 3. 创建数据库路径（使用配置）
    let db_path = &config.database.path;

    println!("[INFO] 初始化存储系统...");
    println!("[INFO] Database: {}\n", db_path);

    // 4. 创建存储仓库
    use qt_rust_demo::repositories::sqlite_storage_repository::SqliteStorageRepository;
    use qt_rust_demo::repositories::storage_repository::StorageRepository;

    let storage_repo = SqliteStorageRepository::new(db_path).await?;
    let storage_repo = Arc::new(storage_repo) as Arc<dyn StorageRepository>;

    println!("[INFO] 数据库初始化完成");

    // 5. 创建共享缓冲区（使用配置的缓冲区大小）
    use qt_rust_demo::pipeline::{ProcessedDataBuffer, SharedBuffer};
    use std::sync::RwLock;

    let shared_buffer: SharedBuffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(
        config.collection.buffer_size,
    )));

    // 6. 创建存储管道（使用配置）
    use qt_rust_demo::pipeline::{StoragePipeline, StoragePipelineConfig};

    let (storage_config, _service_config) =
        StoragePipelineConfig::from_pipeline_config(&config.storage);
    let mut storage_pipeline =
        StoragePipeline::new(storage_config, storage_repo, Arc::clone(&shared_buffer)).await?;

    println!("[INFO] 存储管道创建完成");

    // 7. 启动存储管道
    let _ = storage_pipeline.start();
    println!("[INFO] 存储管道已启动\n");

    // 8. 模拟数据采集（使用配置的采集间隔）
    println!("[INFO] 开始模拟数据采集...");
    println!("[INFO] 采集间隔: {}ms\n", config.collection.interval_ms);

    use qt_rust_demo::models::ProcessedData;
    use qt_rust_demo::models::SensorData;

    let collection_interval = Duration::from_millis(config.collection.interval_ms);
    let total_iterations = 10;

    for i in 1..=total_iterations {
        tokio::time::sleep(collection_interval).await;

        // 模拟采集数据
        let (load, radius, angle) = repository.get_data();
        let sensor_data = SensorData::new(load, radius, angle, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, i);

        // 写入共享缓冲区
        if let Ok(mut buffer) = shared_buffer.write() {
            buffer.push(processed.clone());
        }

        // 如果是危险状态，触发报警回调
        if processed.is_danger {
            println!(
                "  ⚠️  ALARM: Load={:.2}t, Moment={:.2}%",
                processed.current_load, processed.moment_percentage
            );
            storage_pipeline.save_alarm_async(processed.clone());
        }

        // 显示统计信息
        let buffer: SharedBuffer = Arc::clone(&shared_buffer);
        let stats = buffer.read().unwrap().get_stats().clone();
        let queue_len = storage_pipeline.queue_len();
        let last_seq = storage_pipeline.last_stored_sequence();

        println!("[{}s] Stats:", i);
        println!("  - Buffer size: {} items", stats.total_collections);
        println!("  - Storage queue: {} items", queue_len);
        println!("  - Last stored seq: {}", last_seq);
        println!(
            "  - Current: Load={:.2}t, Radius={:.2}m, Angle={:.2}°",
            processed.current_load, processed.working_radius, processed.boom_angle
        );
        println!();
    }

    // 9. 等待存储完成
    println!("[INFO] 等待存储完成...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 10. 停止存储管道
    println!("[INFO] 停止存储管道...");
    storage_pipeline.stop();

    // 11. 显示最终统计
    let buffer: SharedBuffer = Arc::clone(&shared_buffer);
    let stats = buffer.read().unwrap().get_stats().clone();

    println!("\n=== 最终统计 ===");
    println!("总数据条数: {}", stats.total_collections);
    println!(
        "最后存储序列号: {}",
        storage_pipeline.last_stored_sequence()
    );

    println!("\n[INFO] Demo completed successfully");
    println!("[INFO] Check database file: {}", db_path);
    println!("\n提示：可以使用以下命令查看数据库内容：");
    println!(
        "  sqlite3 {} \"SELECT * FROM runtime_data LIMIT 10;\"",
        db_path
    );
    println!("  sqlite3 {} \"SELECT * FROM alarm_records;\"", db_path);

    Ok(())
}
