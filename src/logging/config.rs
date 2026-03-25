use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::Level;

/// 日志级别配置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl LogLevel {
    /// 转换为 tracing::Level
    pub fn to_tracing_level(&self) -> Option<Level> {
        match self {
            LogLevel::Trace => Some(Level::TRACE),
            LogLevel::Debug => Some(Level::DEBUG),
            LogLevel::Info => Some(Level::INFO),
            LogLevel::Warn => Some(Level::WARN),
            LogLevel::Error => Some(Level::ERROR),
            LogLevel::Off => None,
        }
    }

    /// 检查是否应该记录指定级别的日志
    pub fn should_log(&self, level: Level) -> bool {
        match self.to_tracing_level() {
            None => false,
            Some(config_level) => level >= config_level,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// 模块日志级别配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLogLevel {
    /// 模块路径（支持通配符）
    pub module: String,
    /// 日志级别
    pub level: LogLevel,
}

/// 全局日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 默认日志级别
    #[serde(default)]
    pub default_level: LogLevel,
    
    /// 是否输出到控制台
    #[serde(default = "default_true")]
    pub console_output: bool,
    
    /// 是否输出到文件
    #[serde(default)]
    pub file_output: bool,
    
    /// 日志文件路径
    #[serde(default = "default_log_file")]
    pub log_file: String,
    
    /// 各模块的日志级别配置
    #[serde(default)]
    pub modules: Vec<ModuleLogLevel>,
}

fn default_true() -> bool {
    true
}

fn default_log_file() -> String {
    "logs/app.log".to_string()
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            default_level: LogLevel::Info,
            console_output: true,
            file_output: false,
            log_file: default_log_file(),
            modules: vec![],
        }
    }
}

impl LogConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: LogConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 获取指定模块的日志级别
    pub fn get_module_level(&self, module: &str) -> LogLevel {
        // 构建模块匹配缓存
        for config in &self.modules {
            if Self::module_matches(&config.module, module) {
                return config.level;
            }
        }
        self.default_level
    }

    /// 检查模块路径是否匹配
    fn module_matches(pattern: &str, module: &str) -> bool {
        if pattern == module {
            return true;
        }
        
        // 支持通配符匹配
        if pattern.ends_with("::*") {
            let prefix = &pattern[..pattern.len() - 3];
            return module.starts_with(prefix);
        }
        
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return module.starts_with(prefix);
        }
        
        false
    }

    /// 检查指定模块是否应该记录指定级别的日志
    pub fn should_log(&self, module: &str, level: Level) -> bool {
        let module_level = self.get_module_level(module);
        module_level.should_log(level)
    }

    /// 创建示例配置
    pub fn example() -> Self {
        Self {
            default_level: LogLevel::Info,
            console_output: true,
            file_output: true,
            log_file: "logs/app.log".to_string(),
            modules: vec![
                ModuleLogLevel {
                    module: "qt_rust_demo::pipeline::*".to_string(),
                    level: LogLevel::Debug,
                },
                ModuleLogLevel {
                    module: "qt_rust_demo::repositories::*".to_string(),
                    level: LogLevel::Info,
                },
                ModuleLogLevel {
                    module: "qt_rust_demo::pipeline::storage_pipeline".to_string(),
                    level: LogLevel::Trace,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_matches() {
        assert!(LogConfig::module_matches("foo::bar", "foo::bar"));
        assert!(LogConfig::module_matches("foo::*", "foo::bar"));
        assert!(LogConfig::module_matches("foo::*", "foo::bar::baz"));
        assert!(!LogConfig::module_matches("foo::*", "foobar"));
        assert!(LogConfig::module_matches("foo*", "foobar"));
    }

    #[test]
    fn test_get_module_level() {
        let config = LogConfig {
            default_level: LogLevel::Info,
            modules: vec![
                ModuleLogLevel {
                    module: "myapp::pipeline::*".to_string(),
                    level: LogLevel::Debug,
                },
                ModuleLogLevel {
                    module: "myapp::pipeline::storage".to_string(),
                    level: LogLevel::Trace,
                },
            ],
            ..Default::default()
        };

        assert_eq!(config.get_module_level("myapp::other"), LogLevel::Info);
        assert_eq!(config.get_module_level("myapp::pipeline::manager"), LogLevel::Debug);
        assert_eq!(config.get_module_level("myapp::pipeline::storage"), LogLevel::Trace);
    }
}
