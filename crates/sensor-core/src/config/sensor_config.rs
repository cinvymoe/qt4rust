use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 传感器类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    /// 模拟传感器 (AD值)
    Analog,
    /// 数字输入 (开关)
    Digital,
}

/// 单个传感器配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorConfig {
    /// 传感器名称
    pub name: String,
    /// 传感器类型
    #[serde(rename = "type")]
    pub sensor_type: SensorType,
    /// 描述（可选）
    #[serde(default)]
    pub description: Option<String>,
    /// Modbus 从站 ID
    pub slave_id: u8,
    /// Modbus 寄存器地址
    pub register_address: u16,
    /// 寄存器数量
    #[serde(default = "default_register_count")]
    pub register_count: u16,
    /// 数据类型
    #[serde(default = "default_data_type")]
    pub data_type: String,
    /// 字节序
    #[serde(default = "default_byte_order")]
    pub byte_order: String,
}

fn default_register_count() -> u16 {
    1
}
fn default_data_type() -> String {
    "UInt16".to_string()
}
fn default_byte_order() -> String {
    "BigEndian".to_string()
}

/// 传感器配置集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensorsConfig {
    /// 全局配置
    pub global: Option<GlobalConfig>,
    /// Modbus 服务器配置
    pub server: ServerConfig,
    /// 数字输入配置
    pub digital_input: Option<DigitalInputConfig>,
    /// 所有传感器配置 (key = 传感器ID)
    pub sensors: HashMap<String, SensorConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    1000
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 502,
            timeout_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DigitalInputConfig {
    pub source_type: String,
    #[serde(default = "default_toggle_interval")]
    pub toggle_interval_secs: u64,
}

fn default_toggle_interval() -> u64 {
    10
}

impl SensorsConfig {
    /// 从 TOML 字符串解析配置
    pub fn from_toml(toml: &str) -> Result<Self, String> {
        toml::from_str(toml).map_err(|e| format!("配置解析失败: {}", e))
    }

    /// 获取所有模拟传感器配置
    pub fn analog_sensors(&self) -> HashMap<&String, &SensorConfig> {
        self.sensors
            .iter()
            .filter(|(_, c)| c.sensor_type == SensorType::Analog)
            .collect()
    }

    /// 获取所有数字输入传感器配置
    pub fn digital_sensors(&self) -> HashMap<&String, &SensorConfig> {
        self.sensors
            .iter()
            .filter(|(_, c)| c.sensor_type == SensorType::Digital)
            .collect()
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.sensors.is_empty() {
            return Err("至少需要配置一个传感器".to_string());
        }

        // 检查传感器名称唯一性
        let mut names = std::collections::HashSet::new();
        for (id, config) in &self.sensors {
            if !names.insert(&config.name) {
                return Err(format!("传感器名称重复: {}", config.name));
            }
            // 验证 ID 格式
            if id.is_empty() {
                return Err("传感器 ID 不能为空".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sensors_config() {
        let toml = r#"
[global]
mode = "modbus_tcp"

[server]
host = "192.168.1.1"
port = 502

[sensors.main_hook_weight]
name = "主钩重量"
type = "analog"
slave_id = 1
register_address = 0

[sensors.aux_hook_weight]
name = "副钩重量"
type = "analog"
slave_id = 1
register_address = 3

[sensors.main_hook_switch]
name = "主钩开关"
type = "digital"
slave_id = 1
register_address = 10
"#;
        let config = SensorsConfig::from_toml(toml).unwrap();
        assert_eq!(config.sensors.len(), 3);
        assert_eq!(config.server.host, "192.168.1.1");
    }

    #[test]
    fn test_analog_sensors_filter() {
        let mut config = SensorsConfig::default();
        config.sensors.insert(
            "main_hook_weight".to_string(),
            SensorConfig {
                name: "主钩重量".to_string(),
                sensor_type: SensorType::Analog,
                description: None,
                slave_id: 1,
                register_address: 0,
                register_count: 1,
                data_type: "UInt16".to_string(),
                byte_order: "BigEndian".to_string(),
            },
        );
        config.sensors.insert(
            "main_hook_switch".to_string(),
            SensorConfig {
                name: "主钩开关".to_string(),
                sensor_type: SensorType::Digital,
                description: None,
                slave_id: 1,
                register_address: 10,
                register_count: 1,
                data_type: "UInt16".to_string(),
                byte_order: "BigEndian".to_string(),
            },
        );

        let analog = config.analog_sensors();
        assert_eq!(analog.len(), 1);
        assert!(analog.contains_key(&"main_hook_weight".to_string()));
    }

    #[test]
    fn test_validate_empty_sensors() {
        let config = SensorsConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_success() {
        let mut config = SensorsConfig::default();
        config.sensors.insert(
            "test_sensor".to_string(),
            SensorConfig {
                name: "测试传感器".to_string(),
                sensor_type: SensorType::Analog,
                description: None,
                slave_id: 1,
                register_address: 0,
                register_count: 1,
                data_type: "UInt16".to_string(),
                byte_order: "BigEndian".to_string(),
            },
        );
        assert!(config.validate().is_ok());
    }
}
