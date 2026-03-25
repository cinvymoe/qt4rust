use super::config::LogConfig;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// 初始化日志系统
/// 
/// # 参数
/// * `config` - 日志配置，如果为 None 则使用默认配置
/// 
/// # 示例
/// ```no_run
/// use qt_rust_demo::logging::{init_logging, LogConfig};
/// 
/// // 使用默认配置
/// init_logging(None);
/// 
/// // 使用自定义配置
/// let config = LogConfig::from_file("config/logging.toml").ok();
/// init_logging(config);
/// ```
pub fn init_logging(config: Option<LogConfig>) {
    let config = config.unwrap_or_default();
    
    // 构建环境过滤器
    let env_filter = build_env_filter(&config);
    
    // 构建格式化层
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_line_number(true)
        .with_file(true);
    
    // 根据配置决定输出目标
    if config.console_output && !config.file_output {
        // 仅控制台输出
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    } else if config.file_output && !config.console_output {
        // 仅文件输出
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_file)
        {
            let file_layer = fmt::layer()
                .with_writer(std::sync::Arc::new(file))
                .with_target(true)
                .with_line_number(true)
                .with_file(true)
                .with_ansi(false);
            
            tracing_subscriber::registry()
                .with(env_filter)
                .with(file_layer)
                .init();
        } else {
            // 文件打开失败，回退到控制台
            // 注意：此时日志系统未初始化，使用 eprintln 是合理的
            eprintln!("无法打开日志文件: {}, 回退到控制台输出", config.log_file);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        }
    } else if config.console_output && config.file_output {
        // 同时输出到控制台和文件
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_file)
        {
            let file_layer = fmt::layer()
                .with_writer(std::sync::Arc::new(file))
                .with_target(true)
                .with_line_number(true)
                .with_file(true)
                .with_ansi(false);
            
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .with(file_layer)
                .init();
        } else {
            // 注意：此时日志系统未初始化，使用 eprintln 是合理的
            eprintln!("无法打开日志文件: {}, 仅使用控制台输出", config.log_file);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        }
    } else {
        // 都不输出（不推荐）
        tracing_subscriber::registry()
            .with(env_filter)
            .init();
    }
    
    // 保存配置到全局
    let _ = super::set_log_config(config);
}

/// 根据配置构建环境过滤器
fn build_env_filter(config: &LogConfig) -> EnvFilter {
    let mut filter = EnvFilter::new(level_to_str(config.default_level));
    
    // 添加各模块的过滤规则
    for module_config in &config.modules {
        let directive = format!("{}={}", 
            module_config.module.replace("::*", ""),
            level_to_str(module_config.level)
        );
        filter = filter.add_directive(directive.parse().unwrap_or_else(|_| {
            // 注意：此时日志系统未初始化，使用 eprintln 是合理的
            eprintln!("无效的日志指令: {}", directive);
            format!("{}=info", module_config.module).parse().unwrap()
        }));
    }
    
    // 允许通过环境变量 RUST_LOG 覆盖
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        if !rust_log.is_empty() {
            return EnvFilter::new(rust_log);
        }
    }
    
    filter
}

/// 将日志级别转换为字符串
fn level_to_str(level: super::config::LogLevel) -> &'static str {
    match level {
        super::config::LogLevel::Trace => "trace",
        super::config::LogLevel::Debug => "debug",
        super::config::LogLevel::Info => "info",
        super::config::LogLevel::Warn => "warn",
        super::config::LogLevel::Error => "error",
        super::config::LogLevel::Off => "off",
    }
}

/// 快速初始化日志系统（使用默认配置）
pub fn init_default_logging() {
    init_logging(None);
}

/// 从配置文件初始化日志系统
pub fn init_logging_from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = LogConfig::from_file(path)?;
    init_logging(Some(config));
    Ok(())
}
