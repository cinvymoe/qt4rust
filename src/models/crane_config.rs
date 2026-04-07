// src/models/crane_config.rs

use crate::models::rated_load_table::RatedLoadTable;
use crate::models::sensor_calibration::{SensorCalibration, AlarmThresholds};

/// 起重机配置
///
/// 组合传感器标定配置和额定载荷表，作为系统的顶层配置模型
#[derive(Debug, Clone)]
pub struct CraneConfig {
    /// 传感器标定配置
    pub sensor_calibration: SensorCalibration,
    /// 额定载荷表
    pub rated_load_table: RatedLoadTable,
    /// 报警阈值配置
    pub alarm_thresholds: AlarmThresholds,
}

impl CraneConfig {
    /// 验证配置的有效性
    ///
    /// 验证传感器标定配置和额定载荷表是否都有效
    ///
    /// # 返回
    /// - `Ok(())`: 配置有效
    /// - `Err(String)`: 配置无效，包含错误描述
    pub fn validate(&self) -> Result<(), String> {
        // 验证传感器标定配置
        self.sensor_calibration.validate()?;

        // 验证额定载荷表
        self.rated_load_table.validate()?;

        // 验证报警阈值配置
        self.alarm_thresholds.validate()?;

        Ok(())
    }
}

impl Default for CraneConfig {
    /// 提供默认配置
    ///
    /// 使用 SensorCalibration 和 RatedLoadTable 的默认值
    fn default() -> Self {
        Self {
            sensor_calibration: SensorCalibration::default(),
            rated_load_table: RatedLoadTable::default(),
            alarm_thresholds: AlarmThresholds::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CraneConfig::default();

        // 验证默认配置有效
        assert!(config.validate().is_ok());

        // 验证传感器标定配置存在
        assert_eq!(config.sensor_calibration.weight.scale_value, 50.0);
        assert_eq!(config.sensor_calibration.angle.scale_value, 90.0);
        assert_eq!(config.sensor_calibration.radius.scale_value, 20.0);

        // 验证额定载荷表存在
        assert!(!config.rated_load_table.is_empty());
        assert_eq!(config.rated_load_table.len(), 8);
    }

    #[test]
    fn test_validate_success() {
        let config = CraneConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_sensor_calibration() {
        let mut config = CraneConfig::default();

        // 设置无效的传感器标定参数（除零错误）
        config.sensor_calibration.weight.scale_ad = config.sensor_calibration.weight.zero_ad;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("重量传感器"));
    }

    #[test]
    fn test_validate_invalid_load_table() {
        let mut config = CraneConfig::default();

        // 设置无效的载荷表（空表）
        config.rated_load_table.clear();

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("载荷表不能为空"));
    }

    #[test]
    fn test_validate_invalid_angle_thresholds() {
        let mut config = CraneConfig::default();

        // 设置无效的角度阈值（报警值小于预警值）
        config.alarm_thresholds.angle.alarm = 70.0;
        config.alarm_thresholds.angle.warning = 75.0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("angle.alarm"));
    }

    #[test]
    fn test_validate_invalid_moment_thresholds() {
        let mut config = CraneConfig::default();

        // 设置无效的力矩阈值（报警值小于预警值）
        config.alarm_thresholds.moment.alarm_percentage = 80.0;
        config.alarm_thresholds.moment.warning_percentage = 90.0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("moment.alarm_percentage"));
    }

    #[test]
    fn test_validate_multiple_errors() {
        let mut config = CraneConfig::default();

        // 设置多个无效参数，验证会在第一个错误处停止
        config.sensor_calibration.weight.scale_ad = config.sensor_calibration.weight.zero_ad;
        config.rated_load_table.clear();

        let result = config.validate();
        assert!(result.is_err());
        // 应该先检测到传感器标定错误
        assert!(result.unwrap_err().contains("重量传感器"));
    }

    #[test]
    fn test_sensor_calibration_access() {
        let config = CraneConfig::default();

        // 测试可以访问传感器标定配置
        let weight = config.sensor_calibration.convert_weight_ad_to_value(2047.5);
        assert!((weight - 25.0).abs() < 0.1);

        let angle = config.sensor_calibration.convert_angle_ad_to_value(2047.5);
        assert!((angle - 45.0).abs() < 0.1);

        let radius = config.sensor_calibration.convert_radius_ad_to_value(2047.5);
        assert!((radius - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_load_table_access() {
        let config = CraneConfig::default();

        // 测试可以访问额定载荷表（默认臂长20.0）
        let rated_load = config.rated_load_table.get_rated_load(20.0, 5.0);
        assert_eq!(rated_load, 40.0);

        let rated_load = config.rated_load_table.get_rated_load(20.0, 10.0);
        assert_eq!(rated_load, 25.0);
    }

    #[test]
    fn test_warning_and_alarm_checks() {
        let config = CraneConfig::default();

        // 测试角度预警和报警
        assert!(!config.alarm_thresholds.is_angle_warning(60.0));
        assert!(config.alarm_thresholds.is_angle_warning(75.0));
        assert!(config.alarm_thresholds.is_angle_alarm(85.0));

        // 测试力矩预警和报警
        assert!(!config.alarm_thresholds.is_moment_warning(80.0));
        assert!(config.alarm_thresholds.is_moment_warning(90.0));
        assert!(config.alarm_thresholds.is_moment_alarm(100.0));

        // 测试载荷表的力矩预警和报警
        assert!(!config.rated_load_table.is_moment_warning(80.0));
        assert!(config.rated_load_table.is_moment_warning(85.0));
        assert!(config.rated_load_table.is_moment_alarm(95.0));
    }

    #[test]
    fn test_clone() {
        let config = CraneConfig::default();
        let cloned = config.clone();

        // 验证克隆的配置也是有效的
        assert!(cloned.validate().is_ok());

        // 验证克隆的值相同
        assert_eq!(
            config.sensor_calibration.weight.scale_value,
            cloned.sensor_calibration.weight.scale_value
        );
        assert_eq!(config.rated_load_table.len(), cloned.rated_load_table.len());
    }
}
