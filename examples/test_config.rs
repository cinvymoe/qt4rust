// 测试配置加载

use qt_rust_demo::config::pipeline_config::PipelineConfig;

fn main() -> Result<(), String> {
    println!("=== 管道配置加载测试 ===\n");

    // 加载配置
    println!("[INFO] 加载配置文件...");
    let config = PipelineConfig::load();

    // 显示配置参数
    println!("\n[采集管道配置]");
    println!("  采集间隔: {}ms", config.collection.interval_ms);
    println!("  缓冲区大小: {}", config.collection.buffer_size);
    println!("  使用模拟器: {}", config.collection.use_simulator);

    println!("\n[存储管道配置]");
    println!("  存储间隔: {}ms", config.storage.interval_ms);
    println!("  批量大小: {}", config.storage.batch_size);
    println!("  最大重试: {}", config.storage.max_retries);
    println!("  重试延迟: {}ms", config.storage.retry_delay_ms);
    println!("  队列大小: {}", config.storage.max_queue_size);

    println!("\n[数据库配置]");
    println!("  数据库路径: {}", config.database.path);
    println!("  启用 WAL: {}", config.database.enable_wal);
    println!("  连接池大小: {}", config.database.pool_size);

    println!("\n[模拟器配置]");
    println!("  重量传感器:");
    println!("    振幅: {}", config.simulator.weight.amplitude);
    println!("    频率: {}", config.simulator.weight.frequency);
    println!("    偏移: {}", config.simulator.weight.offset);
    println!("    噪声: {}", config.simulator.weight.noise_level);

    println!("  半径传感器:");
    println!("    振幅: {}", config.simulator.radius.amplitude);
    println!("    频率: {}", config.simulator.radius.frequency);
    println!("    偏移: {}", config.simulator.radius.offset);
    println!("    噪声: {}", config.simulator.radius.noise_level);

    println!("  角度传感器:");
    println!("    振幅: {}", config.simulator.angle.amplitude);
    println!("    频率: {}", config.simulator.angle.frequency);
    println!("    偏移: {}", config.simulator.angle.offset);
    println!("    噪声: {}", config.simulator.angle.noise_level);

    println!("\n[性能监控配置]");
    println!("  启用监控: {}", config.monitoring.enable);
    println!("  统计间隔: {}秒", config.monitoring.stats_interval_sec);
    println!("  详细日志: {}", config.monitoring.verbose);

    // 测试 Duration 转换
    println!("\n[Duration 转换测试]");
    println!("  采集间隔: {:?}", config.collection_interval());
    println!("  存储间隔: {:?}", config.storage_interval());
    println!("  重试延迟: {:?}", config.retry_delay());
    println!("  统计间隔: {:?}", config.stats_interval());

    println!("\n[INFO] 配置加载成功！");

    Ok(())
}
