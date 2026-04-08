//! 配置解析模块
//!
//! 提供 TOML 和 CSV 配置文件的解析功能

pub mod toml_parser;
pub mod csv_parser;

use std::path::Path;
use crate::error::HotReloadError;

// 重新导出配置类型（从主项目导入）
// 注意：这些类型定义在主项目中，config-hot-reload crate 只负责解析
pub use qt_rust_demo::models::sensor_calibration::{SensorCalibration, AlarmThresholds};
pub use qt_rust_demo::logging::config::LogConfig;
pub use qt_rust_demo::config::pipeline_config::PipelineConfig;
pub use qt_rust_demo::models::rated_load_table::RatedLoadTable;

// Modbus 配置类型（临时定义，待主项目实现后替换）
use serde::{Deserialize, Serialize};

/// Modbus 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusServerConfig {
    pub address: String,
    pub port: u16,
    pub timeout_ms: u64,
}

impl Default for ModbusServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 502,
            timeout_ms: 1000,
        }
    }
}

/// Modbus 传感器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusSensorConfig {
    pub name: String,
    pub slave_id: u8,
    pub register_address: u16,
    pub register_count: u16,
    pub data_type: String,
}

impl Default for ModbusSensorConfig {
    fn default() -> Self {
        Self {
            name: "sensor".to_string(),
            slave_id: 1,
            register_address: 0,
            register_count: 1,
            data_type: "u16".to_string(),
        }
    }
}

/// Modbus 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusConfig {
    pub server: ModbusServerConfig,
    pub sensors: Vec<ModbusSensorConfig>,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            server: ModbusServerConfig::default(),
            sensors: vec![],
        }
    }
}

/// 配置解析器
///
/// 提供统一的配置文件解析接口
pub struct ConfigParser;

impl ConfigParser {
    /// 解析传感器校准配置
    pub fn parse_sensor_calibration(path: &Path) -> Result<SensorCalibration, HotReloadError> {
        toml_parser::parse_toml(path)
    }
    
    /// 解析报警阈值配置
    pub fn parse_alarm_thresholds(path: &Path) -> Result<AlarmThresholds, HotReloadError> {
        toml_parser::parse_toml(path)
    }
    
    /// 解析日志配置
    pub fn parse_logging_config(path: &Path) -> Result<LogConfig, HotReloadError> {
        toml_parser::parse_toml(path)
    }
    
    /// 解析管道配置
    pub fn parse_pipeline_config(path: &Path) -> Result<PipelineConfig, HotReloadError> {
        toml_parser::parse_toml(path)
    }
    
    /// 解析额定负载表
    pub fn parse_rated_load_table(path: &Path) -> Result<RatedLoadTable, HotReloadError> {
        csv_parser::parse_rated_load_table(path)
    }
    
    /// 解析 Modbus 传感器配置
    pub fn parse_modbus_config(path: &Path) -> Result<ModbusConfig, HotReloadError> {
        toml_parser::parse_toml(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_parse_sensor_calibration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("sensor_calibration.toml");
        
        let content = r#"
[weight]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 50.0
multiplier = 1.0

[angle]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 90.0
multiplier = 1.0

[radius]
zero_ad = 0.0
zero_value = 0.0
scale_ad = 4095.0
scale_value = 20.0
multiplier = 1.0
"#;
        
        fs::write(&config_path, content).unwrap();
        
        let result = ConfigParser::parse_sensor_calibration(&config_path);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.weight.scale_value, 50.0);
        assert_eq!(config.angle.scale_value, 90.0);
        assert_eq!(config.radius.scale_value, 20.0);
    }
    
    #[test]
    fn test_parse_alarm_thresholds() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("alarm_thresholds.toml");
        
        let content = r#"
[moment]
warning_percentage = 85.0
alarm_percentage = 95.0
"#;
        
        fs::write(&config_path, content).unwrap();
        
        let result = ConfigParser::parse_alarm_thresholds(&config_path);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.moment.warning_percentage, 85.0);
        assert_eq!(config.moment.alarm_percentage, 95.0);
    }
    
    #[test]
    fn test_parse_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        
        let content = "invalid toml content [[[";
        fs::write(&config_path, content).unwrap();
        
        let result = ConfigParser::parse_sensor_calibration(&config_path);
        assert!(result.is_err());
        
        match result {
            Err(HotReloadError::ParseError { path: _, reason }) => {
                assert!(reason.contains("TOML"));
            }
            _ => panic!("Expected ParseError"),
        }
    }
    
    #[test]
    fn test_parse_nonexistent_file() {
        let result = ConfigParser::parse_sensor_calibration(Path::new("/nonexistent/file.toml"));
        assert!(result.is_err());
        
        match result {
            Err(HotReloadError::FileRead { .. }) => {}
            _ => panic!("Expected FileRead error"),
        }
    }
}
