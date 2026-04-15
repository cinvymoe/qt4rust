use std::time::Duration;

/// Configuration for the main sensor data pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Interval between sensor readings
    pub read_interval: Duration,
    /// Number of retry attempts on communication failure
    pub max_retries: u32,
    /// Enable debug logging
    pub debug_logging: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            read_interval: Duration::from_millis(100),
            max_retries: 3,
            debug_logging: false,
        }
    }
}

/// Configuration for the storage pipeline.
#[derive(Debug, Clone)]
pub struct StoragePipelineConfig {
    /// Interval between storage writes
    pub storage_interval: Duration,
    /// Number of records to batch before writing
    pub batch_size: usize,
    /// Enable compression for stored data
    pub enable_compression: bool,
}

impl Default for StoragePipelineConfig {
    fn default() -> Self {
        Self {
            storage_interval: Duration::from_secs(5),
            batch_size: 100,
            enable_compression: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();

        assert_eq!(config.read_interval, Duration::from_millis(100));
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug_logging);
    }

    #[test]
    fn test_pipeline_config_custom() {
        let config = PipelineConfig {
            read_interval: Duration::from_millis(50),
            max_retries: 5,
            debug_logging: true,
        };

        assert_eq!(config.read_interval, Duration::from_millis(50));
        assert_eq!(config.max_retries, 5);
        assert!(config.debug_logging);
    }

    #[test]
    fn test_storage_pipeline_config_default() {
        let config = StoragePipelineConfig::default();

        assert_eq!(config.storage_interval, Duration::from_secs(5));
        assert_eq!(config.batch_size, 100);
        assert!(!config.enable_compression);
    }

    #[test]
    fn test_storage_pipeline_config_custom() {
        let config = StoragePipelineConfig {
            storage_interval: Duration::from_secs(10),
            batch_size: 50,
            enable_compression: true,
        };

        assert_eq!(config.storage_interval, Duration::from_secs(10));
        assert_eq!(config.batch_size, 50);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_pipeline_config_clone() {
        let config = PipelineConfig::default();
        let cloned = config.clone();

        assert_eq!(config.read_interval, cloned.read_interval);
        assert_eq!(config.max_retries, cloned.max_retries);
        assert_eq!(config.debug_logging, cloned.debug_logging);
    }

    #[test]
    fn test_storage_pipeline_config_clone() {
        let config = StoragePipelineConfig::default();
        let cloned = config.clone();

        assert_eq!(config.storage_interval, cloned.storage_interval);
        assert_eq!(config.batch_size, cloned.batch_size);
        assert_eq!(config.enable_compression, cloned.enable_compression);
    }
}
