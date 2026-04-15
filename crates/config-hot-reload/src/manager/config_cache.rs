//! 配置缓存
//!
//! 提供线程安全的配置缓存，包含所有配置类型

use qt_rust_demo::config::pipeline_config::PipelineConfig;
use qt_rust_demo::logging::config::LogConfig;
use qt_rust_demo::models::rated_load_table::RatedLoadTable;
use sensor_core::{AlarmThresholds, SensorCalibration};
use std::time::SystemTime;

use crate::parser::ModbusConfig;

/// 配置缓存
///
/// 存储所有配置类型，提供版本号跟踪和时间戳
#[derive(Debug, Clone)]
pub struct ConfigCache {
    /// 传感器校准配置
    pub sensor_calibration: SensorCalibration,

    /// 报警阈值配置
    pub alarm_thresholds: AlarmThresholds,

    /// 日志配置
    pub logging_config: LogConfig,

    /// Modbus 传感器配置
    pub modbus_config: ModbusConfig,

    /// 管道配置
    pub pipeline_config: PipelineConfig,

    /// 额定负载表
    pub rated_load_table: RatedLoadTable,

    /// 配置版本号（每次更新递增）
    pub version: u64,

    /// 最后更新时间
    pub last_updated: SystemTime,
}

impl Default for ConfigCache {
    fn default() -> Self {
        Self {
            sensor_calibration: SensorCalibration::default(),
            alarm_thresholds: AlarmThresholds::default(),
            logging_config: LogConfig::default(),
            modbus_config: ModbusConfig::default(),
            pipeline_config: PipelineConfig::default(),
            rated_load_table: RatedLoadTable::default(),
            version: 0,
            last_updated: SystemTime::now(),
        }
    }
}

impl ConfigCache {
    /// 创建新的配置缓存
    pub fn new() -> Self {
        Self::default()
    }

    /// 递增版本号
    pub fn increment_version(&mut self) {
        self.version = self.version.wrapping_add(1);
        self.last_updated = SystemTime::now();
    }

    /// 获取当前版本号
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 获取最后更新时间
    pub fn last_updated(&self) -> SystemTime {
        self.last_updated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_cache() {
        let cache = ConfigCache::default();
        assert_eq!(cache.version, 0);
        assert_eq!(cache.sensor_calibration.weight.multiplier, 1.0);
    }

    #[test]
    fn test_increment_version() {
        let mut cache = ConfigCache::new();
        assert_eq!(cache.version(), 0);

        cache.increment_version();
        assert_eq!(cache.version(), 1);

        cache.increment_version();
        assert_eq!(cache.version(), 2);
    }

    #[test]
    fn test_version_wrapping() {
        let mut cache = ConfigCache::new();
        cache.version = u64::MAX;

        cache.increment_version();
        assert_eq!(cache.version(), 0); // 应该回绕到 0
    }

    #[test]
    fn test_last_updated_changes() {
        let mut cache = ConfigCache::new();
        let initial_time = cache.last_updated();

        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.increment_version();

        let updated_time = cache.last_updated();
        assert!(updated_time > initial_time);
    }
}
