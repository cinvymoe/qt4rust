// src/config/calibration_manager.rs

use std::fs;
use std::path::Path;
use crane_data_layer::error::{DataError, DataResult};
use crate::models::sensor_calibration::SensorCalibration;

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
    /// 
    /// # 示例
    /// ```
    /// use crate::config::CalibrationManager;
    /// 
    /// let manager = CalibrationManager::new("config/sensor_calibration.toml");
    /// ```
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
        }
    }
    
    /// 加载配置文件
    /// 
    /// 从 TOML 文件读取配置，解析并验证。如果文件不存在，自动创建默认配置。
    /// 
    /// # 返回
    /// - `Ok(SensorCalibration)`: 加载成功，返回配置对象
    /// - `Err(DataError)`: 加载失败，返回错误信息
    /// 
    /// # 错误处理
    /// - 文件不存在：创建默认配置文件
    /// - 文件读取失败：返回 IoError
    /// - TOML 解析失败：返回 SerializationError
    /// - 配置验证失败：返回 ValidationError
    /// 
    /// # 示例
    /// ```
    /// let manager = CalibrationManager::new("config/sensor_calibration.toml");
    /// match manager.load() {
    ///     Ok(config) => println!("配置加载成功"),
    ///     Err(e) => eprintln!("配置加载失败: {}", e),
    /// }
    /// ```
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
        let content = fs::read_to_string(path)
            .map_err(|e| DataError::IoError(format!("无法读取配置文件 {}: {}", self.config_path, e)))?;
        
        // 解析 TOML
        tracing::debug!("解析 TOML 配置");
        let config: SensorCalibration = toml::from_str(&content)
            .map_err(|e| DataError::SerializationError(format!("TOML 解析失败: {}", e)))?;
        
        // 验证配置
        tracing::debug!("验证配置参数");
        config.validate()
            .map_err(|e| DataError::ValidationError(e))?;
        
        tracing::info!("配置加载成功: {}", self.config_path);
        Ok(config)
    }
    
    /// 保存配置到文件
    /// 
    /// 验证配置有效性后，序列化为 TOML 格式并写入文件。
    /// 自动添加注释和时间戳，确保配置文件可读性。
    /// 
    /// # 参数
    /// - `config`: 要保存的配置对象
    /// 
    /// # 返回
    /// - `Ok(())`: 保存成功
    /// - `Err(DataError)`: 保存失败，返回错误信息
    /// 
    /// # 错误处理
    /// - 配置验证失败：返回 ValidationError
    /// - TOML 序列化失败：返回 SerializationError
    /// - 目录创建失败：返回 IoError
    /// - 文件写入失败：返回 IoError
    /// 
    /// # 示例
    /// ```
    /// let manager = CalibrationManager::new("config/sensor_calibration.toml");
    /// let config = SensorCalibration::default();
    /// match manager.save(&config) {
    ///     Ok(_) => println!("配置保存成功"),
    ///     Err(e) => eprintln!("配置保存失败: {}", e),
    /// }
    /// ```
    pub fn save(&self, config: &SensorCalibration) -> DataResult<()> {
        // 验证配置
        tracing::debug!("验证配置参数");
        config.validate()
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
             # 标定步骤:\n\
             # 1. 零点标定: 传感器无负载时，记录 AD 值和对应的物理值\n\
             # 2. 满量程标定: 传感器满负载时，记录 AD 值和对应的物理值\n\
             # 3. 验证: 使用中间值验证转换公式的准确性\n\
             #\n\
             # 注意事项:\n\
             # - 确保 scale_ad 不等于 zero_ad，否则会导致除零错误\n\
             # - 角度预警值和报警值必须在 0-90 范围内\n\
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
        fs::write(&self.config_path, content)
            .map_err(|e| DataError::IoError(format!("无法写入配置文件 {}: {}", self.config_path, e)))?;
        
        tracing::info!("配置已保存: {}", self.config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    /// 创建临时测试目录
    fn setup_test_dir() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("sensor_calibration.toml");
        let config_path_str = config_path.to_str().unwrap().to_string();
        (temp_dir, config_path_str)
    }
    
    #[test]
    fn test_new() {
        let manager = CalibrationManager::new("config/test.toml");
        assert_eq!(manager.config_path, "config/test.toml");
    }
    
    #[test]
    fn test_load_creates_default_if_not_exists() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);
        
        // 文件不存在时应该创建默认配置
        let result = manager.load();
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.weight_zero_ad, 0.0);
        assert_eq!(config.weight_scale_ad, 4095.0);
        
        // 验证文件已创建
        assert!(Path::new(&config_path).exists());
    }
    
    #[test]
    fn test_save_and_load() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);
        
        // 创建自定义配置
        let mut config = SensorCalibration::default();
        config.weight_zero_ad = 100.0;
        config.weight_scale_ad = 4000.0;
        config.angle_warning_value = 70.0;
        
        // 保存配置
        let save_result = manager.save(&config);
        assert!(save_result.is_ok());
        
        // 加载配置
        let load_result = manager.load();
        assert!(load_result.is_ok());
        
        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.weight_zero_ad, 100.0);
        assert_eq!(loaded_config.weight_scale_ad, 4000.0);
        assert_eq!(loaded_config.angle_warning_value, 70.0);
    }
    
    #[test]
    fn test_save_invalid_config() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);
        
        // 创建无效配置（scale_ad 等于 zero_ad）
        let mut config = SensorCalibration::default();
        config.weight_scale_ad = config.weight_zero_ad;
        
        // 保存应该失败
        let result = manager.save(&config);
        assert!(result.is_err());
        
        if let Err(DataError::ValidationError(msg)) = result {
            assert!(msg.contains("重量传感器"));
        } else {
            panic!("Expected ValidationError");
        }
    }
    
    #[test]
    fn test_load_invalid_toml() {
        let (_temp_dir, config_path) = setup_test_dir();
        
        // 写入无效的 TOML 内容
        fs::write(&config_path, "invalid toml content [[[").unwrap();
        
        let manager = CalibrationManager::new(&config_path);
        let result = manager.load();
        
        assert!(result.is_err());
        if let Err(DataError::SerializationError(msg)) = result {
            assert!(msg.contains("TOML 解析失败"));
        } else {
            panic!("Expected SerializationError");
        }
    }
    
    #[test]
    fn test_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("subdir/config/sensor_calibration.toml");
        let config_path_str = config_path.to_str().unwrap();
        
        let manager = CalibrationManager::new(config_path_str);
        let config = SensorCalibration::default();
        
        // 保存应该自动创建目录
        let result = manager.save(&config);
        assert!(result.is_ok());
        
        // 验证目录和文件都已创建
        assert!(config_path.parent().unwrap().exists());
        assert!(config_path.exists());
    }
    
    #[test]
    fn test_round_trip_preserves_data() {
        let (_temp_dir, config_path) = setup_test_dir();
        let manager = CalibrationManager::new(&config_path);
        
        // 创建配置并保存
        let original_config = SensorCalibration {
            weight_zero_ad: 100.0,
            weight_zero_value: 5.0,
            weight_scale_ad: 4000.0,
            weight_scale_value: 45.0,
            angle_zero_ad: 200.0,
            angle_zero_value: 10.0,
            angle_scale_ad: 3800.0,
            angle_scale_value: 80.0,
            radius_zero_ad: 150.0,
            radius_zero_value: 2.0,
            radius_scale_ad: 3900.0,
            radius_scale_value: 18.0,
            angle_warning_value: 70.0,
            angle_alarm_value: 80.0,
            moment_warning_percentage: 85.0,
            moment_alarm_percentage: 95.0,
        };
        
        manager.save(&original_config).unwrap();
        
        // 加载并验证
        let loaded_config = manager.load().unwrap();
        
        assert_eq!(loaded_config.weight_zero_ad, original_config.weight_zero_ad);
        assert_eq!(loaded_config.weight_zero_value, original_config.weight_zero_value);
        assert_eq!(loaded_config.weight_scale_ad, original_config.weight_scale_ad);
        assert_eq!(loaded_config.weight_scale_value, original_config.weight_scale_value);
        assert_eq!(loaded_config.angle_warning_value, original_config.angle_warning_value);
        assert_eq!(loaded_config.angle_alarm_value, original_config.angle_alarm_value);
        assert_eq!(loaded_config.moment_warning_percentage, original_config.moment_warning_percentage);
        assert_eq!(loaded_config.moment_alarm_percentage, original_config.moment_alarm_percentage);
    }
}
