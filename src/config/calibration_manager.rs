// src/config/calibration_manager.rs

use crate::models::sensor_calibration::SensorCalibration;
use crane_data_layer::error::{DataError, DataResult};
use std::fs;
use std::path::Path;

/// 标定配置管理器
///
/// 负责加载、保存和管理传感器标定配置文件（TOML 格式）
pub struct CalibrationManager {
    /// 配置文件路径
    config_path: String,
}

impl CalibrationManager {
    /// 创建新的标定配置管理器
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径（相对或绝对路径）
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }

    /// 加载配置文件
    ///
    /// 从 TOML 文件读取配置，解析并验证。如果文件不存在，自动创建默认配置。
    pub fn load(&self) -> DataResult<SensorCalibration> {
        let path = Path::new(&self.config_path);

        // 如果文件不存在，创建默认配置
        if !path.exists() {
            tracing::info!("配置文件不存在，创建默认配置: {}", self.config_path);
            let default_config = SensorCalibration::default();
            self.save(&default_config)?;
            return Ok(default_config);
        }

        // 读取文件
        tracing::debug!("读取配置文件: {}", self.config_path);
        let content = fs::read_to_string(path).map_err(|e| {
            DataError::IoError(format!("无法读取配置文件 {}: {}", self.config_path, e))
        })?;

        // 解析 TOML
        tracing::debug!("解析 TOML 配置");
        let config: SensorCalibration = toml::from_str(&content)
            .map_err(|e| DataError::SerializationError(format!("TOML 解析失败: {}", e)))?;

        // 验证配置
        tracing::debug!("验证配置参数");
        config
            .validate()
            .map_err(|e| DataError::ValidationError(e))?;

        tracing::info!("配置加载成功: {}", self.config_path);
        Ok(config)
    }

    /// 保存配置到文件
    ///
    /// 验证配置有效性后，序列化为 TOML 格式并写入文件。
    pub fn save(&self, config: &SensorCalibration) -> DataResult<()> {
        // 验证配置
        tracing::debug!("验证配置参数");
        config
            .validate()
            .map_err(|e| DataError::ValidationError(e))?;

        // 序列化为 TOML
        tracing::debug!("序列化配置为 TOML");
        let toml_string = toml::to_string_pretty(config)
            .map_err(|e| DataError::SerializationError(format!("TOML 序列化失败: {}", e)))?;

        // 添加注释和时间戳
        let content = format!(
            "# 起重机传感器标定配置文件\n\
             # 自动生成于: {}\n\
             #\n\
             # ========== 传感器标定说明 ==========\n\
             # 转换公式: 物理值 = 零点物理值 + (AD值 - 零点AD) × (放大物理值 - 零点物理值) / (放大AD - 零点AD)\n\
             #\n\
             # 标定步骤：\n\
             # 1. 零点标定: 传感器无负载时，记录 AD 值和对应的物理值\n\
             # 2. 满量程标定: 传感器满负载时，记录 AD 值和对应的物理值\n\
             # 3. 验证: 使用中间值验证转换公式的准确性\n\
             #\n\
             # 注意事项：\n\
             # - 确保 scale_ad 不等于 zero_ad，否则会导致除零错误\n\n\
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

        tracing::info!("配置已保存: {}", self.config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("sensor_calibration.toml");
        let config_path_str = config_path.to_str().unwrap().to_string();
        (temp_dir, config_path_str)
    }

    #[test]
    fn test_load_creates_default_if_not_exists() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);

        let result = manager.load();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.weight.zero_ad, 0.0);
        assert_eq!(config.weight.scale_ad, 4095.0);

        assert!(Path::new(&config_path).exists());
    }

    #[test]
    fn test_save_and_load() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);

        let mut config = SensorCalibration::default();
        config.weight.zero_ad = 100.0;
        config.weight.scale_ad = 4000.0;
        config.angle.zero_value = 10.0;

        let save_result = manager.save(&config);
        assert!(save_result.is_ok());

        let load_result = manager.load();
        assert!(load_result.is_ok());

        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.weight.zero_ad, 100.0);
        assert_eq!(loaded_config.weight.scale_ad, 4000.0);
        assert_eq!(loaded_config.angle.zero_value, 10.0);
    }

    #[test]
    fn test_save_invalid_config() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);

        let mut config = SensorCalibration::default();
        config.weight.scale_ad = config.weight.zero_ad;

        let result = manager.save(&config);
        assert!(result.is_err());

        if let Err(DataError::ValidationError(msg)) = result {
            assert!(msg.contains("重量传感器"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_round_trip_preserves_data() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);

        let original_config = SensorCalibration {
            weight: crate::models::sensor_calibration::SensorCalibrationParams {
                zero_ad: 100.0,
                zero_value: 5.0,
                scale_ad: 4000.0,
                scale_value: 45.0,
            },
            angle: crate::models::sensor_calibration::SensorCalibrationParams {
                zero_ad: 200.0,
                zero_value: 10.0,
                scale_ad: 3800.0,
                scale_value: 80.0,
            },
            radius: crate::models::sensor_calibration::SensorCalibrationParams {
                zero_ad: 150.0,
                zero_value: 2.0,
                scale_ad: 3900.0,
                scale_value: 18.0,
            },
        };

        manager.save(&original_config).unwrap();

        let loaded_config = manager.load().unwrap();

        assert_eq!(loaded_config.weight.zero_ad, original_config.weight.zero_ad);
        assert_eq!(
            loaded_config.weight.zero_value,
            original_config.weight.zero_value
        );
        assert_eq!(loaded_config.angle.zero_ad, original_config.angle.zero_ad);
        assert_eq!(
            loaded_config.radius.scale_value,
            original_config.radius.scale_value
        );
    }
}
