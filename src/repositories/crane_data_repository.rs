// 起重机数据仓库

use crate::config::config_manager::ConfigManager;
use crate::data_sources::SensorDataSource;
use crate::models::crane_config::CraneConfig;
use crate::models::SensorData;
use crane_data_layer::error::DataResult;
use std::sync::{Arc, Mutex};

/// 起重机数据仓库
pub struct CraneDataRepository {
    /// 传感器数据源
    sensor_source: Arc<Mutex<SensorDataSource>>,

    /// 数据缓存
    cache: Arc<Mutex<Option<SensorData>>>,

    /// 配置管理器
    config_manager: Arc<ConfigManager>,
}

impl CraneDataRepository {
    /// 创建新的数据仓库
    ///
    /// # 参数
    /// - `config_manager`: 配置管理器的 Arc 引用
    ///
    /// # 示例
    /// ```
    /// let config_manager = Arc::new(ConfigManager::new()?);
    /// let repo = CraneDataRepository::new(config_manager);
    /// ```
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        // 根据配置文件创建传感器数据源
        let mut sensor_source = match SensorDataSource::from_config() {
            Ok(source) => {
                tracing::info!("传感器数据源初始化成功");
                source
            }
            Err(e) => {
                tracing::error!("传感器数据源初始化失败: {}", e);
                tracing::warn!("使用默认模拟传感器");
                SensorDataSource::new()
            }
        };
        
        // 连接传感器（对于 Modbus 传感器必须先连接）
        if let Err(e) = sensor_source.connect() {
            tracing::error!("传感器连接失败: {}", e);
            tracing::warn!("将使用模拟传感器");
            sensor_source = SensorDataSource::new();
        } else {
            tracing::info!("传感器连接成功");
        }
        
        Self {
            sensor_source: Arc::new(Mutex::new(sensor_source)),
            cache: Arc::new(Mutex::new(None)),
            config_manager,
        }
    }

    /// 获取最新传感器数据
    pub fn get_latest_sensor_data(&self) -> Result<SensorData, String> {
        let sensor_source = self
            .sensor_source
            .lock()
            .map_err(|e| format!("Failed to lock sensor source: {}", e))?;

        let data = sensor_source.read_data()?;

        // 更新缓存
        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some(data.clone());
        }

        Ok(data)
    }

    /// 获取缓存的数据
    #[allow(dead_code)]
    pub fn get_cached_data(&self) -> Option<SensorData> {
        self.cache.lock().ok()?.clone()
    }

    /// 获取当前配置
    ///
    /// 委托给 ConfigManager 获取当前配置。
    ///
    /// # 返回
    /// - `Ok(CraneConfig)`: 返回当前配置
    /// - `Err(DataError)`: 配置未加载或获取失败
    ///
    /// # 示例
    /// ```
    /// let config = repo.get_config()?;
    /// let weight = config.sensor_calibration.convert_weight_ad_to_value(2048.0);
    /// ```
    pub fn get_config(&self) -> DataResult<CraneConfig> {
        self.config_manager.get_config()
    }

    /// 重新加载配置
    ///
    /// 委托给 ConfigManager 重新加载配置。
    /// 从文件重新读取配置，验证后更新缓存，并通知所有观察者。
    /// 如果加载失败，会自动回滚到旧配置。
    ///
    /// # 返回
    /// - `Ok(CraneConfig)`: 重新加载成功，返回新配置
    /// - `Err(DataError)`: 重新加载失败，已回滚到旧配置
    ///
    /// # 示例
    /// ```
    /// match repo.reload_config() {
    ///     Ok(config) => println!("配置重载成功"),
    ///     Err(e) => eprintln!("配置重载失败: {:?}", e),
    /// }
    /// ```
    pub fn reload_config(&self) -> DataResult<CraneConfig> {
        self.config_manager.reload_config()
    }

    /// 克隆 Repository（用于跨线程共享）
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

        // 初始缓存为空
        assert!(repo.get_cached_data().is_none());

        // 读取数据后缓存应该有值
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
