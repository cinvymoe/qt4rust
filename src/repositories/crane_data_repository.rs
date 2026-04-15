// 起重机数据仓库

use crate::config::config_manager::ConfigManager;
use crate::models::crane_config::CraneConfig;
use crate::models::SensorData;
use crane_data_layer::error::DataResult;
use sensor_core::SensorSource;
use sensor_simulator::prelude::SimulatedDataSource;
use std::sync::{Arc, Mutex};

pub enum SensorSourceEnum {
    Simulated(SimulatedDataSource),
    #[allow(dead_code)]
    Modbus(Box<modbus_tcp::ModbusDataSource>),
}

impl SensorSourceEnum {
    fn read_all(&self) -> Result<(f64, f64, f64, bool, bool), String> {
        match self {
            SensorSourceEnum::Simulated(ds) => {
                SensorSource::read_all(ds).map_err(|e| e.to_string())
            }
            SensorSourceEnum::Modbus(ds) => {
                SensorSource::read_all(ds.as_ref()).map_err(|e| e.to_string())
            }
        }
    }
}

pub struct CraneDataRepository {
    sensor_source: Arc<Mutex<SensorSourceEnum>>,
    cache: Arc<Mutex<Option<SensorData>>>,
    config_manager: Arc<ConfigManager>,
}

impl CraneDataRepository {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        let sensor_source = Self::create_sensor_source();

        Self {
            sensor_source: Arc::new(Mutex::new(sensor_source)),
            cache: Arc::new(Mutex::new(None)),
            config_manager,
        }
    }

    fn create_sensor_source() -> SensorSourceEnum {
        if !std::path::Path::new("config/modbus_sensors.toml").exists() {
            tracing::info!("使用模拟传感器（未检测到 Modbus 配置文件）");
            return SensorSourceEnum::Simulated(SimulatedDataSource::new());
        }

        tracing::info!("检测到 Modbus 配置文件，尝试加载...");
        let config_content = match std::fs::read_to_string("config/modbus_sensors.toml") {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("读取 Modbus 配置失败: {}，使用模拟传感器", e);
                return SensorSourceEnum::Simulated(SimulatedDataSource::new());
            }
        };

        let mut ds = match modbus_tcp::ModbusDataSource::from_config(&config_content) {
            Ok(ds) => ds,
            Err(e) => {
                tracing::warn!("Modbus 配置解析失败: {}，使用模拟传感器", e);
                return SensorSourceEnum::Simulated(SimulatedDataSource::new());
            }
        };

        match ds.connect() {
            Ok(_) => {
                tracing::info!("Modbus 传感器连接成功");
                SensorSourceEnum::Modbus(Box::new(ds))
            }
            Err(e) => {
                tracing::warn!("Modbus 连接失败: {}，使用模拟传感器", e);
                SensorSourceEnum::Simulated(SimulatedDataSource::new())
            }
        }
    }

    pub fn get_latest_sensor_data(&self) -> Result<SensorData, String> {
        let sensor_source = self
            .sensor_source
            .lock()
            .map_err(|e| format!("Failed to lock sensor source: {}", e))?;

        let (ad1, ad2, ad3, di0, di1) = sensor_source.read_all()?;

        let data = SensorData::new(ad1, ad2, ad3, di0, di1);

        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some(data.clone());
        }

        Ok(data)
    }

    #[allow(dead_code)]
    pub fn get_cached_data(&self) -> Option<SensorData> {
        self.cache.lock().ok()?.clone()
    }

    pub fn get_config(&self) -> DataResult<CraneConfig> {
        self.config_manager.get_config()
    }

    pub fn reload_config(&self) -> DataResult<CraneConfig> {
        self.config_manager.reload_config()
    }

    #[allow(dead_code)]
    pub fn clone_arc(&self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            sensor_source: Arc::clone(&self.sensor_source),
            cache: Arc::clone(&self.cache),
            config_manager: Arc::clone(&self.config_manager),
        }))
    }
}

impl Default for CraneDataRepository {
    fn default() -> Self {
        let config_manager = Arc::new(ConfigManager::default());
        Self::new(config_manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_latest_sensor_data() {
        let repo = CraneDataRepository::default();
        let result = repo.get_latest_sensor_data();

        assert!(result.is_ok());
    }

    #[test]
    fn test_cache() {
        let repo = CraneDataRepository::default();

        assert!(repo.get_cached_data().is_none());

        let _ = repo.get_latest_sensor_data();
        assert!(repo.get_cached_data().is_some());
    }

    #[test]
    fn test_get_config() {
        let repo = CraneDataRepository::default();
        let result = repo.get_config();

        assert!(result.is_ok(), "应该能获取配置");

        let config = result.unwrap();
        assert!(config.sensor_calibration.weight.scale_ad > 0.0);
        assert!(config.rated_load_table.len() > 0);
    }

    #[test]
    fn test_reload_config() {
        let repo = CraneDataRepository::default();
        let result = repo.reload_config();

        assert!(result.is_ok(), "配置重载应该成功");
    }
}
