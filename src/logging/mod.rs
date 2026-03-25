/// 全局日志管理模块
/// 
/// 提供按模块/文件单独控制日志级别的功能

pub mod config;
pub mod filter;

pub use config::{LogConfig, ModuleLogLevel};
pub use filter::{init_logging, init_default_logging, init_logging_from_file};

use std::sync::OnceLock;
use tracing::Level;

/// 全局日志配置
static LOG_CONFIG: OnceLock<LogConfig> = OnceLock::new();

/// 获取全局日志配置
pub fn get_log_config() -> &'static LogConfig {
    LOG_CONFIG.get_or_init(|| {
        LogConfig::from_file("config/logging.toml")
            .unwrap_or_else(|_| LogConfig::default())
    })
}

/// 设置全局日志配置
pub fn set_log_config(config: LogConfig) -> Result<(), LogConfig> {
    LOG_CONFIG.set(config)
}

/// 检查指定模块是否应该记录指定级别的日志
pub fn should_log(module: &str, level: Level) -> bool {
    get_log_config().should_log(module, level)
}

/// 便捷宏：按模块控制的日志宏
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        if $crate::logging::should_log(module_path!(), tracing::Level::TRACE) {
            tracing::trace!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if $crate::logging::should_log(module_path!(), tracing::Level::DEBUG) {
            tracing::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if $crate::logging::should_log(module_path!(), tracing::Level::INFO) {
            tracing::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if $crate::logging::should_log(module_path!(), tracing::Level::WARN) {
            tracing::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if $crate::logging::should_log(module_path!(), tracing::Level::ERROR) {
            tracing::error!($($arg)*);
        }
    };
}
