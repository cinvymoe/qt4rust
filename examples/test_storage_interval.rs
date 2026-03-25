// 测试存储间隔配置是否生效

use std::sync::Arc;
use std::time::Duration;

// 引入必要的模块
use qt_rust_demo::config::pipeline_config::PipelineConfig;
use qt_rust_demo::repositories::CraneDataRepository;
use qt_rust_demo::pipeline::PipelineManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("存储间隔配置测试");
    println!("========================================\n");
    
    // 1. 加载配置文件
    println!("[1] 加载配置文件...");
    let config = PipelineConfig::load();
    
    println!("✓ 配置加载成功");
    println!("  - 采集间隔: {}ms", config.collection.interval_ms);
    println!("  - 存储间隔: {}ms", config.storage.interval_ms);
    println!("  - 批量大小: {}", config.storage.batch_size);
    println!("  - 最大重试: {}", config.storage.max_retries);
    println!("  - 队列容量: {}", config.storage.max_queue_size);
    println!();
    
    // 2. 创建数据仓库
    println!("[2] 创建数据仓库...");
    let repository = Arc::new(CraneDataRepository::default());
    println!("✓ 数据仓库创建成功\n");
    
    // 3. 创建管道管理器（带存储支持）
    println!("[3] 创建管道管理器（带存储支持）...");
    let db_path = "test_storage_interval.db";
    
    // 删除旧数据库
    let _ = std::fs::remove_file(db_path);
    
    let mut manager: PipelineManager = 
        PipelineManager::new_with_storage(
            repository,
            db_path,
        ).await?;
    
    println!("✓ 管道管理器创建成功\n");
    
    // 4. 启动所有管道
    println!("[4] 启动所有管道...");
    manager.start_all();
    println!("✓ 管道已启动\n");
    
    // 5. 监控存储队列变化
    println!("[5] 监控存储队列变化（持续 20 秒）...");
    println!("时间(s) | 队列长度 | 最后存储序列号");
    println!("--------|----------|----------------");
    
    let start_time = std::time::Instant::now();
    let mut last_stored_seq = 0u64;
    let mut storage_count = 0;
    
    for i in 0..20 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let queue_len = manager.get_storage_queue_len().unwrap_or(0);
        let stored_seq = manager.get_last_stored_sequence().unwrap_or(0);
        
        // 检测是否发生了存储
        if stored_seq > last_stored_seq {
            storage_count += 1;
            println!("{:7} | {:8} | {:16} ← 存储发生！", 
                     i + 1, queue_len, stored_seq);
            last_stored_seq = stored_seq;
        } else {
            println!("{:7} | {:8} | {:16}", 
                     i + 1, queue_len, stored_seq);
        }
    }
    
    println!();
    
    // 6. 统计结果
    println!("[6] 测试结果统计");
    println!("  - 运行时间: {}s", start_time.elapsed().as_secs());
    println!("  - 存储次数: {}", storage_count);
    println!("  - 配置间隔: {}ms", config.storage.interval_ms);
    let expected_count: u64 = 20000 / config.storage.interval_ms;
    println!("  - 预期次数: {} 次", expected_count);
    println!();
    
    // 验证结果
    let tolerance: u64 = 2; // 允许 ±2 次误差
    
    if storage_count >= expected_count.saturating_sub(tolerance) 
        && storage_count <= expected_count + tolerance {
        println!("✓ 测试通过！存储间隔配置生效");
        println!("  实际存储次数 ({}) 符合预期 ({} ±{})", 
                 storage_count, expected_count, tolerance);
    } else {
        println!("✗ 测试失败！存储间隔配置可能未生效");
        println!("  实际存储次数 ({}) 不符合预期 ({})", 
                 storage_count, expected_count);
    }
    println!();
    
    // 7. 停止管道
    println!("[7] 停止管道...");
    manager.stop_all();
    println!("✓ 管道已停止\n");
    
    // 8. 清理测试数据库
    println!("[8] 清理测试数据库...");
    let _ = std::fs::remove_file(db_path);
    println!("✓ 清理完成\n");
    
    println!("========================================");
    println!("测试完成");
    println!("========================================");
    
    Ok(())
}
