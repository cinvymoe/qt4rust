/// 使用配置文件的日志系统演示
///
/// 运行方式:
/// ```bash
/// cargo run --example logging_with_config_file
/// ```
use qt_rust_demo::logging::init_logging_from_file;
use tracing::{debug, error, info, trace, warn};

fn main() {
    // 从配置文件初始化日志系统
    match init_logging_from_file("config/logging.toml") {
        Ok(_) => info!("日志系统初始化成功"),
        Err(e) => {
            // 日志系统未初始化，使用 eprintln 是合理的
            eprintln!("日志系统初始化失败: {}", e);
            eprintln!("使用默认配置");
            qt_rust_demo::logging::init_default_logging();
        }
    }

    info!("=== 应用程序启动 ===");

    // 模拟不同模块的日志输出
    simulate_pipeline_module();
    simulate_repository_module();
    simulate_data_source_module();

    info!("=== 应用程序结束 ===");
}

fn simulate_pipeline_module() {
    info!("Pipeline 模块开始工作");
    debug!("初始化数据采集管道");
    trace!("配置参数: interval=100ms, buffer_size=1000");

    for i in 1..=3 {
        debug!("采集数据批次 {}", i);
        trace!("数据详情: batch={}, count={}", i, i * 10);
    }

    info!("Pipeline 模块工作完成");
}

fn simulate_repository_module() {
    info!("Repository 模块开始工作");
    debug!("连接到数据库");

    match save_data_to_db() {
        Ok(_) => info!("数据保存成功"),
        Err(e) => error!("数据保存失败: {}", e),
    }

    info!("Repository 模块工作完成");
}

fn simulate_data_source_module() {
    info!("DataSource 模块开始工作");
    debug!("读取传感器数据");

    let sensor_data = read_sensor_data();
    info!("传感器数据: {:?}", sensor_data);

    if sensor_data.temperature > 80.0 {
        warn!("温度过高: {:.1}°C", sensor_data.temperature);
    }

    info!("DataSource 模块工作完成");
}

fn save_data_to_db() -> Result<(), String> {
    debug!("执行 SQL: INSERT INTO ...");
    trace!("SQL 参数: id=1, value=42.0");
    Ok(())
}

#[derive(Debug)]
struct SensorData {
    temperature: f64,
    #[allow(dead_code)]
    humidity: f64,
    #[allow(dead_code)]
    pressure: f64,
}

fn read_sensor_data() -> SensorData {
    trace!("从传感器读取原始数据");
    debug!("解析传感器数据");

    SensorData {
        temperature: 75.5,
        humidity: 60.0,
        pressure: 1013.25,
    }
}
