// 起重机数据仓库

use crate::config::config_manager::ConfigManager;
use crate::models::crane_config::CraneConfig;
use crate::models::SensorData;
use crane_data_layer::error::DataResult;
use sensor_core::{init_builtin_sources, SensorSource, SensorSourceFactory};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct CraneDataRepository {
    sensor_source: Arc<Mutex<Box<dyn SensorSource>>>,
    cache: Arc<Mutex<Option<SensorData>>>,
    config_manager: Arc<ConfigManager>,
}

impl CraneDataRepository {
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        init_builtin_sources();

        let config_path = PathBuf::from("config/modbus_sensors.toml");
        let sensor_source = SensorSourceFactory::create_from_config(&config_path);

        Self {
            sensor_source: Arc::new(Mutex::new(sensor_source)),
            cache: Arc::new(Mutex::new(None)),
            config_manager,
        }
    }

    pub fn get_latest_sensor_data(&self) -> Result<SensorData, String> {
        let sensor_source = self
            .sensor_source
            .lock()
            .map_err(|e| format!("Failed to lock sensor source: {}", e))?;

        let (ad1, ad2, ad3, di0, di1) = sensor_source.read_all().map_err(|e| e.to_string())?;

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
