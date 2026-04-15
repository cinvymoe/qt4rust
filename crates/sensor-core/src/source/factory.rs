use std::path::Path;

use super::analog_simulator::SimulatedAnalogSource;
use super::combined::CombinedSensorSource;
use super::digital::DigitalInputConfig;
use super::digital_simulator::SimulatedDigitalInput;
use super::digital_simulator::SimulatedDigitalInputFactory;
use super::registry::DigitalInputRegistry;
use crate::{AnalogSource, DigitalInputSource, SensorResult, SensorSource};
use modbus_tcp::ModbusDataSource;

struct ModbusAnalogSource {
    inner: ModbusDataSource,
}

impl ModbusAnalogSource {
    fn from_config(config_path: &Path) -> SensorResult<Box<dyn AnalogSource>> {
        let config_content = std::fs::read_to_string(config_path)
            .map_err(|e| crate::SensorError::ConfigError(format!("读取配置失败: {}", e)))?;

        let mut source = ModbusDataSource::from_config(&config_content)
            .map_err(|e| crate::SensorError::ConfigError(format!("Modbus 配置错误: {:?}", e)))?;

        source
            .connect()
            .map_err(|e| crate::SensorError::InitError(format!("Modbus 连接失败: {:?}", e)))?;

        Ok(Box::new(Self { inner: source }))
    }
}

impl AnalogSource for ModbusAnalogSource {
    fn read(&self) -> SensorResult<(f64, f64, f64)> {
        self.inner
            .read_all()
            .map_err(|e| crate::SensorError::ReadError(format!("Modbus 读取失败: {:?}", e)))
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    fn source_name(&self) -> &str {
        "ModbusAnalogSource"
    }
}

pub struct SensorSourceFactory;

impl SensorSourceFactory {
    pub fn create_from_config(config_path: &Path) -> Box<dyn SensorSource> {
        let analog = Self::create_analog_source(config_path);
        let digital = Self::create_digital_source(config_path);

        Box::new(CombinedSensorSource::new(analog, digital))
    }

    fn create_analog_source(config_path: &Path) -> Box<dyn AnalogSource> {
        if !config_path.exists() {
            tracing::info!("未检测到 Modbus 配置，使用模拟器");
            return Box::new(SimulatedAnalogSource::new());
        }

        match ModbusAnalogSource::from_config(config_path) {
            Ok(source) => {
                tracing::info!("Modbus 模拟量源连接成功");
                source
            }
            Err(e) => {
                tracing::warn!("Modbus 连接失败: {}，回退到模拟器", e);
                Box::new(SimulatedAnalogSource::new())
            }
        }
    }

    fn create_digital_source(config_path: &Path) -> Box<dyn DigitalInputSource> {
        let config = Self::parse_digital_config(config_path).unwrap_or_else(|e| {
            tracing::debug!("解析数字输入配置失败: {}，使用默认配置", e);
            DigitalInputConfig::default()
        });

        DigitalInputRegistry::global()
            .create(&config)
            .unwrap_or_else(|e| {
                tracing::warn!("创建数字输入源失败: {}，使用模拟器", e);
                Box::new(SimulatedDigitalInput::new(10))
            })
    }

    fn parse_digital_config(config_path: &Path) -> SensorResult<DigitalInputConfig> {
        let content = std::fs::read_to_string(config_path)
            .map_err(|e| crate::SensorError::ConfigError(format!("读取配置失败: {}", e)))?;

        let value: toml::Value = toml::from_str(&content)
            .map_err(|e| crate::SensorError::ConfigError(format!("解析配置失败: {}", e)))?;

        let digital_section = value.get("digital_input").ok_or_else(|| {
            crate::SensorError::ConfigError("缺少 [digital_input] 配置段".to_string())
        })?;

        let config: DigitalInputConfig = digital_section.clone().try_into().map_err(|e| {
            crate::SensorError::ConfigError(format!("解析 digital_input 配置失败: {}", e))
        })?;

        Ok(config)
    }
}

pub fn init_builtin_sources() {
    DigitalInputRegistry::global().register(std::sync::Arc::new(SimulatedDigitalInputFactory));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_from_config_no_file() {
        init_builtin_sources();

        let source = SensorSourceFactory::create_from_config(Path::new("/nonexistent/config.toml"));

        let result = source.read_all().unwrap();
        assert!(result.0 >= 0.0 && result.0 <= 4095.0);
    }

    #[test]
    fn test_parse_digital_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[digital_input]
source_type = "simulator"
toggle_interval_secs = 5
"#
        )
        .unwrap();

        let config = SensorSourceFactory::parse_digital_config(file.path()).unwrap();
        assert_eq!(config.source_type, "simulator");
        assert_eq!(config.toggle_interval_secs, 5);
    }
}
