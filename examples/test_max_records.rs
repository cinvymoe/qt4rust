// 测试数据库最大记录条数功能
//
// 功能说明：
// 1. 测试 max_records 限制是否生效
// 2. 测试 purge_threshold 触发机制
// 3. 验证旧数据是否被正确清理
//
// 使用方法：
// cargo run --example test_max_records

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use qt_rust_demo::models::{ProcessedData, SensorData};
use qt_rust_demo::repositories::sqlite_storage_repository::SqliteStorageRepository;
use qt_rust_demo::repositories::storage_repository::StorageRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n========================================");
    println!("  数据库最大记录条数功能测试");
    println!("========================================\n");

    // 测试场景 1: max_records = 10, purge_threshold = 0 (默认 11)
    println!("【测试 1】max_records=10, purge_threshold=0 (默认 11)");
    test_max_records_scenario(10, 0, 15).await?;

    // 测试场景 2: max_records = 20, purge_threshold = 25
    println!("\n【测试 2】max_records=20, purge_threshold=25");
    test_max_records_scenario(20, 25, 30).await?;

    // 测试场景 3: max_records = 0 (不限制)
    println!("\n【测试 3】max_records=0 (不限制)");
    test_unlimited_records().await?;

    println!("\n========================================");
    println!("  所有测试完成！");
    println!("========================================\n");

    Ok(())
}

/// 测试指定的 max_records 场景
async fn test_max_records_scenario(
    max_records: usize,
    purge_threshold: usize,
    insert_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建测试数据库
    let db_path = format!("test_max_records_{}.db", max_records);

    // 删除旧数据库
    let _ = std::fs::remove_file(&db_path);

    // 创建 Repository
    let repo: Arc<dyn StorageRepository> = Arc::new(SqliteStorageRepository::new(&db_path).await?);

    println!(
        "  配置: max_records={}, purge_threshold={}",
        max_records, purge_threshold
    );
    println!("  插入 {} 条记录...", insert_count);

    // 插入测试数据
    for i in 1..=insert_count {
        let sensor_data = SensorData::new(
            10.0 + i as f64,
            8.0 + (i % 5) as f64,
            60.0 + (i % 10) as f64,
        );
        let processed = ProcessedData::from_sensor_data(sensor_data, i as u64);

        repo.save_runtime_data_batch(&vec![processed]).await?;

        // 每插入 5 条记录，执行一次清理检查
        if i % 5 == 0 {
            let purged = repo.purge_old_records(max_records, purge_threshold).await?;
            if purged > 0 {
                println!("    [第 {} 条] 清理了 {} 条旧记录", i, purged);
            }
        }
    }

    // 最后再执行一次清理
    let purged = repo.purge_old_records(max_records, purge_threshold).await?;
    if purged > 0 {
        println!("  最终清理: 删除了 {} 条旧记录", purged);
    }

    // 查询当前记录数
    let count = repo.get_runtime_data_count().await?;
    println!("  ✓ 当前记录数: {}", count);

    // 验证结果
    if count <= max_records as i64 {
        println!(
            "  ✓ 测试通过: 记录数 {} <= max_records {}",
            count, max_records
        );
    } else {
        println!(
            "  ✗ 测试失败: 记录数 {} > max_records {}",
            count, max_records
        );
    }

    // 查询最早和最晚的记录
    let records = repo.get_runtime_data_range(0, 1).await?;
    if let Some(first) = records.first() {
        println!("  最早记录: sequence={}", first.sequence_number);
    }

    let records = repo
        .get_runtime_data_range(count.saturating_sub(1) as i64, 1)
        .await?;
    if let Some(last) = records.first() {
        println!("  最晚记录: sequence={}", last.sequence_number);
    }

    // 清理测试数据库
    drop(repo);
    sleep(Duration::from_millis(100)).await;
    let _ = std::fs::remove_file(&db_path);

    Ok(())
}

/// 测试不限制记录数
async fn test_unlimited_records() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "test_unlimited.db";

    // 删除旧数据库
    let _ = std::fs::remove_file(db_path);

    // 创建 Repository
    let repo: Arc<dyn StorageRepository> = Arc::new(SqliteStorageRepository::new(db_path).await?);

    println!("  配置: max_records=0 (不限制)");
    println!("  插入 50 条记录...");

    // 插入 50 条记录
    for i in 1..=50 {
        let sensor_data = SensorData::new(10.0 + i as f64, 8.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, i as u64);
        repo.save_runtime_data_batch(&vec![processed]).await?;
    }

    // 尝试清理（应该不删除任何记录）
    let purged = repo.purge_old_records(0, 0).await?;
    println!("  清理结果: 删除了 {} 条记录", purged);

    // 查询当前记录数
    let count = repo.get_runtime_data_count().await?;
    println!("  ✓ 当前记录数: {}", count);

    if count == 50 && purged == 0 {
        println!("  ✓ 测试通过: max_records=0 时不清理数据");
    } else {
        println!("  ✗ 测试失败: 预期 50 条记录，实际 {} 条", count);
    }

    // 清理测试数据库
    drop(repo);
    sleep(Duration::from_millis(100)).await;
    let _ = std::fs::remove_file(db_path);

    Ok(())
}
