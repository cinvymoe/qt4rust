// 管道配置加载模块

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// 管道系统完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub collection: CollectionConfig,
    pub storage: StorageConfig,
    pub display: DisplayConfig,
    pub database: DatabaseConfig,
    pub simulator: SimulatorConfig,
    pub monitoring: MonitoringConfig,
}

/// 数据采集管道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// 采集间隔（毫秒）
    pub interval_ms: u64,

    /// 缓冲区大小
    pub buffer_size: usize,

    /// 是否使用模拟传感器
    pub use_simulator: bool,
}

/// 存储管道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储间隔（毫秒）
    pub interval_ms: u64,

    /// 批量存储大小
    pub batch_size: usize,

    /// 失败重试次数
    pub max_retries: u32,

    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,

    /// 存储队列最大容量
    pub max_queue_size: usize,

    /// 数据库最大记录条数（0 表示不限制）
    pub max_records: usize,

    /// 清理阈值：超过此值时才执行清理（0 表示使用默认值 max_records * 1.1）
    pub purge_threshold: usize,

    /// 报警记录最大条数（0 表示不限制）
    pub alarm_max_records: usize,

    /// 报警清理阈值（0 表示使用默认值 alarm_max_records * 1.1）
    pub alarm_purge_threshold: usize,
}

/// 显示管道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// 采集间隔（毫秒）
    pub interval_ms: u64,

    /// 管道大小
    pub pipeline_size: usize,

    /// 每次采集数量
    pub batch_size: usize,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            interval_ms: 500,
            pipeline_size: 10,
            batch_size: 1,
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库文件路径
    pub path: String,

    /// 是否启用 WAL 模式
    pub enable_wal: bool,

    /// 连接池大小
    pub pool_size: u32,
}

/// 传感器模拟器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    pub weight: SensorSimConfig,
    pub radius: SensorSimConfig,
    pub angle: SensorSimConfig,
}

/// 单个传感器模拟配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorSimConfig {
    pub amplitude: f64,
    pub frequency: f64,
    pub offset: f64,
    pub noise_level: f64,
}

/// 性能监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 是否启用监控
    pub enable: bool,

    /// 统计间隔（秒）
    pub stats_interval_sec: u64,

    /// 是否打印详细日志
    pub verbose: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            collection: CollectionConfig::default(),
            storage: StorageConfig::default(),
            display: DisplayConfig::default(),
            database: DatabaseConfig::default(),
            simulator: SimulatorConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            interval_ms: 100,
            buffer_size: 1000,
            use_simulator: true,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            batch_size: 10,
            max_retries: 3,
            retry_delay_ms: 100,
            max_queue_size: 1000,
            max_records: 0,
            purge_threshold: 0,
            alarm_max_records: 0,
            alarm_purge_threshold: 0,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "crane_data.db".to_string(),
            enable_wal: true,
            pool_size: 5,
        }
    }
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            weight: SensorSimConfig {
                amplitude: 5.0,
                frequency: 0.5,
                offset: 15.0,
                noise_level: 0.1,
            },
            radius: SensorSimConfig {
                amplitude: 3.0,
                frequency: 0.3,
                offset: 8.0,
                noise_level: 0.05,
            },
            angle: SensorSimConfig {
                amplitude: 10.0,
                frequency: 0.2,
                offset: 60.0,
                noise_level: 0.5,
            },
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable: true,
            stats_interval_sec: 5,
            verbose: false,
        }
    }
}

impl PipelineConfig {
    /// 从 TOML 文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: PipelineConfig =
            toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.validate()?;

        Ok(config)
    }

    /// 从默认路径加载配置
    /// 优先级：./config/pipeline_config.toml > 默认配置
    pub fn load() -> Self {
        let config_path = "config/pipeline_config.toml";

        match Self::from_file(config_path) {
            Ok(config) => {
                tracing::info!("Loaded config from: {}", config_path);
                config
            }
            Err(e) => {
                tracing::warn!("Failed to load config: {}", e);
                tracing::info!("Using default configuration");
                Self::default()
            }
        }
    }

    /// 验证配置参数
    fn validate(&self) -> Result<(), String> {
        // 验证采集间隔
        if self.collection.interval_ms < 50 {
            return Err("Collection interval must be >= 50ms".to_string());
        }

        // 验证缓冲区大小
        if self.collection.buffer_size == 0 {
            return Err("Buffer size must be > 0".to_string());
        }

        // 验证存储间隔
        if self.storage.interval_ms < self.collection.interval_ms {
            return Err("Storage interval should be >= collection interval".to_string());
        }

        // 验证批量大小
        if self.storage.batch_size == 0 {
            return Err("Batch size must be > 0".to_string());
        }

        // 验证队列大小
        if self.storage.max_queue_size == 0 {
            return Err("Queue size must be > 0".to_string());
        }

        // 验证数据库路径
        if self.database.path.is_empty() {
            return Err("Database path cannot be empty".to_string());
        }

        Ok(())
    }

    /// 获取采集间隔 Duration
    pub fn collection_interval(&self) -> Duration {
        Duration::from_millis(self.collection.interval_ms)
    }

    /// 获取存储间隔 Duration
    pub fn storage_interval(&self) -> Duration {
        Duration::from_millis(self.storage.interval_ms)
    }

    /// 获取重试延迟 Duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.storage.retry_delay_ms)
    }

    /// 获取监控统计间隔 Duration
    pub fn stats_interval(&self) -> Duration {
        Duration::from_secs(self.monitoring.stats_interval_sec)
    }

    /// 获取显示间隔 Duration
    pub fn display_interval(&self) -> Duration {
        Duration::from_millis(self.display.interval_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PipelineConfig::default();
        assert_eq!(config.collection.interval_ms, 100);
        assert_eq!(config.storage.batch_size, 10);
        assert!(config.collection.use_simulator);
    }

    #[test]
    fn test_validate_success() {
        let config = PipelineConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_collection_interval_too_small() {
        let mut config = PipelineConfig::default();
        config.collection.interval_ms = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_storage_interval_too_small() {
        let mut config = PipelineConfig::default();
        config.storage.interval_ms = 50;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_duration_conversions() {
        let config = PipelineConfig::default();
        assert_eq!(config.collection_interval(), Duration::from_millis(100));
        assert_eq!(config.storage_interval(), Duration::from_millis(1000));
        assert_eq!(config.retry_delay(), Duration::from_millis(100));
    }
}
