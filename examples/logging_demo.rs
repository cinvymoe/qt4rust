use qt_rust_demo::logging::config::LogLevel;
/// 日志系统演示示例
///
/// 运行方式:
/// ```bash
/// cargo run --example logging_demo
/// ```
use qt_rust_demo::logging::{init_logging, LogConfig, ModuleLogLevel};
use tracing::{debug, error, info, trace, warn};

fn main() {
    // 创建自定义配置
    let config = LogConfig {
        default_level: LogLevel::Info,
        console_output: true,
        file_output: false,
        log_file: "logs/demo.log".to_string(),
        modules: vec![ModuleLogLevel {
            module: "logging_demo".to_string(),
            level: LogLevel::Trace,
        }],
    };

    // 初始化日志系统
    init_logging(Some(config));

    info!("=== 日志系统演示开始 ===");

    // 演示不同级别的日志
    demo_log_levels();

    // 演示带参数的日志
    demo_log_with_params();

    // 演示结构化日志
    demo_structured_logging();

    info!("=== 日志系统演示结束 ===");
}

fn demo_log_levels() {
    info!("--- 演示不同日志级别 ---");

    trace!("这是 TRACE 级别日志（最详细）");
    debug!("这是 DEBUG 级别日志（调试信息）");
    info!("这是 INFO 级别日志（一般信息）");
    warn!("这是 WARN 级别日志（警告信息）");
    error!("这是 ERROR 级别日志（错误信息）");
}

fn demo_log_with_params() {
    info!("--- 演示带参数的日志 ---");

    let sensor_id = 42;
    let temperature = 25.5;
    let status = "正常";

    debug!("传感器 {} 读取数据", sensor_id);
    info!("温度: {:.1}°C, 状态: {}", temperature, status);

    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    trace!("原始数据: {:?}", data);
}

fn demo_structured_logging() {
    info!("--- 演示结构化日志 ---");

    // 使用 tracing 的结构化字段
    info!(
        sensor_id = 42,
        temperature = 25.5,
        humidity = 60.0,
        "传感器数据更新"
    );

    warn!(error_code = 1001, retry_count = 3, "数据采集失败，正在重试");

    error!(
        module = "storage_pipeline",
        error = "Connection timeout",
        "数据库连接失败"
    );
}
