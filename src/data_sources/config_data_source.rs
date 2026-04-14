// src/data_sources/config_data_source.rs

use crate::config::alarm_threshold_manager::AlarmThresholdManager;
use crate::config::calibration_manager::CalibrationManager;
use crate::config::load_table_manager::LoadTableManager;
use crate::models::crane_config::CraneConfig;
use crane_data_layer::error::{DataError, DataResult};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// 配置缓存结构
#[derive(Clone)]
struct ConfigCache {
    config: CraneConfig,
    version: u64,
    timestamp: SystemTime,
}

/// 配置数据源
///
/// 负责加载、缓存和重新加载起重机配置参数
pub struct ConfigDataSource {
    calibration_manager: CalibrationManager,
    load_table_manager: LoadTableManager,
    alarm_threshold_manager: AlarmThresholdManager,
    cached_config: Arc<RwLock<Option<ConfigCache>>>,
}

impl ConfigDataSource {
    /// 创建新的配置数据源
    ///
    /// 使用默认配置文件路径：
    /// - 传感器标定: `config/sensor_calibration.toml`
    /// - 额定载荷表: `config/rated_load_table.csv`
    /// - 报警阈值: `config/alarm_thresholds.toml`
    pub fn new() -> Self {
        Self {
            calibration_manager: CalibrationManager::new("config/sensor_calibration.toml"),
            load_table_manager: LoadTableManager::new("config/rated_load_table.csv"),
            alarm_threshold_manager: AlarmThresholdManager::new("config/alarm_thresholds.toml"),
            cached_config: Arc::new(RwLock::new(None)),
        }
    }

    /// 加载配置（启动时调用）
    ///
    /// 从配置文件读取传感器标定参数和额定载荷表，验证后缓存到内存。
    /// 如果配置文件不存在，会自动创建默认配置。
    ///
    /// # 返回
    /// - `Ok(CraneConfig)`: 加载成功，返回配置对象
    /// - `Err(DataError)`: 加载失败，返回错误信息
    ///
    /// # 示例
    /// ```
    /// let config_source = ConfigDataSource::new();
    /// let config = config_source.load()?;
    /// ```
    pub fn load(&self) -> DataResult<CraneConfig> {
        tracing::info!("开始加载配置...");

        // 加载传感器标定配置
        let sensor_calibration = self.calibration_manager.load().map_err(|e| {
            tracing::error!("加载传感器标定配置失败: {:?}", e);
            e
        })?;

        tracing::debug!("传感器标定配置加载成功");

        // 加载额定载荷表
        let rated_load_table = self.load_table_manager.load().map_err(|e| {
            tracing::error!("加载额定载荷表失败: {:?}", e);
            e
        })?;

        tracing::debug!("额定载荷表加载成功");

        // 加载报警阈值配置
        let alarm_thresholds = self.alarm_threshold_manager.load().map_err(|e| {
            tracing::error!("加载报警阈值配置失败: {:?}", e);
            e
        })?;

        tracing::debug!("报警阈值配置加载成功");

        // 组合配置
        let config = CraneConfig {
            sensor_calibration,
            rated_load_table,
            alarm_thresholds,
        };

        // 验证配置
        config.validate().map_err(|e| {
            tracing::error!("配置验证失败: {}", e);
            DataError::ValidationError(e)
        })?;

        // 更新缓存（使用写锁）
        let mut cache = self.cached_config.write().map_err(|e| {
            let err_msg = format!("无法获取写锁: {}", e);
            tracing::error!("{}", err_msg);
            DataError::CacheError(err_msg)
        })?;

        let version = cache.as_ref().map(|c| c.version + 1).unwrap_or(1);
        *cache = Some(ConfigCache {
            config: config.clone(),
            version,
            timestamp: SystemTime::now(),
        });

        tracing::info!("配置加载成功，版本: {}", version);
        Ok(config)
    }

    /// 重新加载配置（运行时调用，带回滚机制）
    ///
    /// 从文件重新读取配置，如果加载失败则回滚到旧配置。
    /// 这确保了配置重载失败时系统仍能正常运行。
    ///
    /// # 返回
    /// - `Ok(CraneConfig)`: 重新加载成功，返回新配置
    /// - `Err(DataError)`: 重新加载失败，已回滚到旧配置
    ///
    /// # 示例
    /// ```
    /// match config_source.reload() {
    ///     Ok(config) => println!("配置重载成功"),
    ///     Err(e) => eprintln!("配置重载失败，已回滚: {:?}", e),
    /// }
    /// ```
    pub fn reload(&self) -> DataResult<CraneConfig> {
        tracing::info!("开始重新加载配置...");

        // 保存旧配置以便回滚
        let old_cache = {
            let cache = self.cached_config.read().map_err(|e| {
                let err_msg = format!("无法获取读锁: {}", e);
                tracing::error!("{}", err_msg);
                DataError::CacheError(err_msg)
            })?;
            cache.clone()
        };

        // 尝试加载新配置
        match self.load() {
            Ok(config) => {
                tracing::info!("配置重新加载成功，新配置已生效");
                Ok(config)
            }
            Err(e) => {
                // 加载失败，回滚到旧配置
                if let Some(old) = old_cache {
                    let mut cache = self.cached_config.write().map_err(|e2| {
                        let err_msg = format!("回滚失败: {}", e2);
                        tracing::error!("{}", err_msg);
                        DataError::CacheError(err_msg)
                    })?;
                    *cache = Some(old);
                    tracing::warn!("配置加载失败，已回滚到旧配置: {:?}", e);
                } else {
                    tracing::error!("配置加载失败且无旧配置可回滚: {:?}", e);
                }
                Err(e)
            }
        }
    }

    /// 获取缓存的配置（使用读锁，高并发性能更好）
    ///
    /// 从内存缓存中读取配置，不会触发文件 I/O。
    /// 使用读锁允许多个线程同时读取配置。
    ///
    /// # 返回
    /// - `Ok(CraneConfig)`: 返回缓存的配置
    /// - `Err(DataError::NotFound)`: 配置未加载
    ///
    /// # 示例
    /// ```
    /// let config = config_source.get_cached_config()?;
    /// let weight = config.sensor_calibration.convert_weight_ad_to_value(2048.0);
    /// ```
    pub fn get_cached_config(&self) -> DataResult<CraneConfig> {
        let cache = self.cached_config.read().map_err(|e| {
            let err_msg = format!("无法获取读锁: {}", e);
            tracing::error!("{}", err_msg);
            DataError::CacheError(err_msg)
        })?;

        cache.as_ref().map(|c| c.config.clone()).ok_or_else(|| {
            tracing::warn!("尝试获取配置，但配置未加载");
            DataError::NotFound("配置未加载".to_string())
        })
    }

    /// 获取配置版本号
    ///
    /// 版本号在每次配置重载时递增，可用于检测配置是否已更新。
    ///
    /// # 返回
    /// - `Ok(u64)`: 当前配置版本号
    /// - `Err(DataError::NotFound)`: 配置未加载
    pub fn get_config_version(&self) -> DataResult<u64> {
        let cache = self.cached_config.read().map_err(|e| {
            let err_msg = format!("无法获取读锁: {}", e);
            tracing::error!("{}", err_msg);
            DataError::CacheError(err_msg)
        })?;

        cache.as_ref().map(|c| c.version).ok_or_else(|| {
            tracing::warn!("尝试获取配置版本，但配置未加载");
            DataError::NotFound("配置未加载".to_string())
        })
    }

    /// 获取配置加载时间戳
    ///
    /// 返回配置最后一次加载的时间，可用于显示配置更新时间。
    ///
    /// # 返回
    /// - `Ok(SystemTime)`: 配置加载时间戳
    /// - `Err(DataError::NotFound)`: 配置未加载
    pub fn get_config_timestamp(&self) -> DataResult<SystemTime> {
        let cache = self.cached_config.read().map_err(|e| {
            let err_msg = format!("无法获取读锁: {}", e);
            tracing::error!("{}", err_msg);
            DataError::CacheError(err_msg)
        })?;

        cache.as_ref().map(|c| c.timestamp).ok_or_else(|| {
            tracing::warn!("尝试获取配置时间戳，但配置未加载");
            DataError::NotFound("配置未加载".to_string())
        })
    }
}

impl Default for ConfigDataSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_data_source_new() {
        let config_source = ConfigDataSource::new();

        // 验证初始状态：配置未加载
        assert!(config_source.get_cached_config().is_err());
        assert!(config_source.get_config_version().is_err());
        assert!(config_source.get_config_timestamp().is_err());
    }

    #[test]
    fn test_config_data_source_load() {
        let config_source = ConfigDataSource::new();

        // 加载配置
        let result = config_source.load();
        assert!(result.is_ok(), "配置加载应该成功");

        let config = result.unwrap();
        assert!(config.sensor_calibration.weight.scale_ad > 0.0);
        assert!(config.rated_load_table.len() > 0);
    }

    #[test]
    fn test_config_data_source_cache() {
        let config_source = ConfigDataSource::new();

        // 加载配置
        config_source.load().expect("配置加载失败");

        // 获取缓存的配置
        let cached_config = config_source.get_cached_config();
        assert!(cached_config.is_ok(), "应该能获取缓存的配置");

        // 获取版本号
        let version = config_source.get_config_version();
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), 1, "首次加载版本号应为 1");

        // 获取时间戳
        let timestamp = config_source.get_config_timestamp();
        assert!(timestamp.is_ok());
    }

    #[test]
    fn test_config_data_source_reload() {
        let config_source = ConfigDataSource::new();

        // 首次加载
        config_source.load().expect("首次加载失败");
        let version1 = config_source.get_config_version().unwrap();

        // 重新加载
        let result = config_source.reload();
        assert!(result.is_ok(), "配置重载应该成功");

        let version2 = config_source.get_config_version().unwrap();
        assert_eq!(version2, version1 + 1, "重载后版本号应递增");
    }

    #[test]
    fn test_config_data_source_thread_safety() {
        use std::thread;

        let config_source = Arc::new(ConfigDataSource::new());
        config_source.load().expect("配置加载失败");

        // 创建多个线程同时读取配置
        let mut handles = vec![];

        for _ in 0..10 {
            let config_source_clone = Arc::clone(&config_source);
            let handle = thread::spawn(move || {
                let config = config_source_clone.get_cached_config();
                assert!(config.is_ok(), "多线程读取配置应该成功");
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().expect("线程执行失败");
        }
    }
}
