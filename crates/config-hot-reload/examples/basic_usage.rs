use config_hot_reload::prelude::*;
use config_hot_reload::subscribers::SharedConfigRefs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 配置热加载基本使用示例 ===\n");

    // 1. 创建共享配置引用（用于在组件和订阅者之间共享配置）
    println!("1. 创建共享配置引用...");
    let shared_refs = SharedConfigRefs::default();
    println!("   ✓ 默认配置已加载\n");

    // 2. 创建配置管理器
    println!("2. 创建配置管理器...");
    let config_dir = std::path::PathBuf::from("config");
    if !config_dir.exists() {
        println!("   ⚠ 配置目录不存在，跳过热加载演示");
        println!("   提示: 在项目根目录创建 config/ 目录并放入配置文件即可使用热加载功能\n");
        println!("=== 示例结束 ===");
        return Ok(());
    }

    let mut manager = HotReloadConfigManager::new(config_dir)?;
    println!("   ✓ 配置管理器已创建\n");

    // 3. 注册配置订阅者
    println!("3. 注册配置变更订阅者...");
    use config_hot_reload::subscribers::register_all_subscribers;
    register_all_subscribers(&mut manager, &shared_refs).await;
    println!("   ✓ 已注册 5 个订阅者:\n");
    println!("     - PipelineConfigSubscriber (管道配置)");
    println!("     - DataProcessingSubscriber (传感器校准 + 负载表)");
    println!("     - AlarmDetectionSubscriber (报警阈值)");
    println!("     - LoggingConfigSubscriber (日志配置)");
    println!("     - SensorDataSourceSubscriber (Modbus配置)\n");

    // 4. 启动热加载服务
    println!("4. 启动热加载服务...");
    println!("   提示: 修改 config/ 目录下的配置文件将自动触发重载\n");

    // 注意: start() 会阻塞，实际使用中应在独立的 tokio 任务中运行
    // 这里仅演示 API 调用，不实际启动
    println!("   （示例中不实际启动文件监控）\n");

    // 5. 手动重载配置
    println!("5. 手动重载所有配置...");
    match manager.reload_all().await {
        Ok(()) => println!("   ✓ 所有配置重载成功\n"),
        Err(e) => println!("   ✗ 重载失败: {}\n", e),
    }

    // 6. 获取当前配置快照
    println!("6. 获取当前配置快照...");
    let snapshot = manager.get_config_snapshot().await;
    println!("   ✓ 配置版本: {}\n", snapshot.version);

    // 7. 通过共享引用读取当前配置
    println!("7. 通过共享引用读取配置...");
    {
        let cal = shared_refs.sensor_calibration.read().unwrap();
        println!("   重量传感器: scale_value = {} 吨", cal.weight.scale_value);
        println!("   角度传感器: scale_value = {} 度", cal.angle.scale_value);
        println!("   半径传感器: scale_value = {} 米", cal.radius.scale_value);
    }
    {
        let thresholds = shared_refs.alarm_thresholds.read().unwrap();
        println!("   预警阈值: {}%", thresholds.moment.warning_percentage);
        println!("   报警阈值: {}%", thresholds.moment.alarm_percentage);
    }
    {
        let pipeline = shared_refs.pipeline_config.read().unwrap();
        println!("   采集间隔: {}ms", pipeline.collection.interval_ms);
        println!("   存储间隔: {}ms", pipeline.storage.interval_ms);
    }

    println!("\n=== 示例结束 ===");
    Ok(())
}
