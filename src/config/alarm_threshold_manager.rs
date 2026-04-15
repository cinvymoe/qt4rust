// src/config/alarm_threshold_manager.rs

use crane_data_layer::error::{DataError, DataResult};
use sensor_core::AlarmThresholds;
use std::fs;
use std::path::Path;

/// 报警阈值配置管理器
///
/// 负责加载、保存和管理报警阈值配置文件（TOML 格式）
pub struct AlarmThresholdManager {
    /// 配置文件路径
    config_path: String,
}

impl AlarmThresholdManager {
    /// 创建新的报警阈值配置管理器
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }

    /// 加载配置文件
    pub fn load(&self) -> DataResult<AlarmThresholds> {
        let path = Path::new(&self.config_path);

        // 如果文件不存在，创建默认配置
        if !path.exists() {
            tracing::info!("报警阈值配置文件不存在，创建默认配置: {}", self.config_path);
            let default_config = AlarmThresholds::default();
            self.save(&default_config)?;
            return Ok(default_config);
        }

        // 读取文件
        tracing::debug!("读取报警阈值配置文件: {}", self.config_path);
        let content = fs::read_to_string(path).map_err(|e| {
            DataError::IoError(format!("无法读取配置文件 {}: {}", self.config_path, e))
        })?;

        // 解析 TOML
        tracing::debug!("解析 TOML 配置");
        let config: AlarmThresholds = toml::from_str(&content)
            .map_err(|e| DataError::SerializationError(format!("TOML 解析失败: {}", e)))?;

        // 验证配置
        tracing::debug!("验证报警阈值配置");
        config
            .validate()
            .map_err(|e| DataError::ValidationError(e))?;

        tracing::info!("报警阈值配置加载成功: {}", self.config_path);
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self, config: &AlarmThresholds) -> DataResult<()> {
        // 验证配置
        tracing::debug!("验证报警阈值配置");
        config
            .validate()
            .map_err(|e| DataError::ValidationError(e))?;

        // 序列化为 TOML
        tracing::debug!("序列化配置为 TOML");
        let toml_string = toml::to_string_pretty(config)
            .map_err(|e| DataError::SerializationError(format!("TOML 序列化失败: {}", e)))?;

        // 添加注释和时间戳
        let content = format!(
            "# 起重机报警阈值配置文件\n\
             # 自动生成于: {}\n\
             #\n\
             # 说明：\n\
             # 本文件包含所有报警阈值参数，用于判断起重机是否处于危险状态\n\
             # 这些阈值可以根据实际工况动态调整\n\
             #\n\
             # 注意事项：\n\
             # - 力矩百分比阈值必须在 0-100 范围内\n\
             # - 报警值必须大于等于预警值\n\n\
             {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            toml_string
        );

        // 确保目录存在
        if let Some(parent) = Path::new(&self.config_path).parent() {
            tracing::debug!("确保配置目录存在: {:?}", parent);
            fs::create_dir_all(parent)
                .map_err(|e| DataError::IoError(format!("无法创建配置目录: {}", e)))?;
        }

        // 写入文件
        tracing::debug!("写入配置文件: {}", self.config_path);
        fs::write(&self.config_path, content).map_err(|e| {
            DataError::IoError(format!("无法写入配置文件 {}: {}", self.config_path, e))
        })?;

        tracing::info!("报警阈值配置已保存: {}", self.config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("alarm_thresholds.toml");
        let config_path_str = config_path.to_str().unwrap().to_string();
        (temp_dir, config_path_str)
    }

    #[test]
    fn test_load_creates_default_if_not_exists() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = AlarmThresholdManager::new(&config_path);

        let result = manager.load();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.moment.warning_percentage, 90.0);
        assert_eq!(config.moment.alarm_percentage, 100.0);

        assert!(Path::new(&config_path).exists());
    }

    #[test]
    fn test_save_and_load() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = AlarmThresholdManager::new(&config_path);

        let mut config = AlarmThresholds::default();
        config.moment.warning_percentage = 85.0;
        config.moment.alarm_percentage = 95.0;

        let save_result = manager.save(&config);
        assert!(save_result.is_ok());

        let load_result = manager.load();
        assert!(load_result.is_ok());

        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.moment.warning_percentage, 85.0);
        assert_eq!(loaded_config.moment.alarm_percentage, 95.0);
    }

    #[test]
    fn test_save_invalid_config() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = AlarmThresholdManager::new(&config_path);

        let mut config = AlarmThresholds::default();
        config.moment.alarm_percentage = 80.0; // 小于 warning
        config.moment.warning_percentage = 90.0;

        let result = manager.save(&config);
        assert!(result.is_err());
    }
}
