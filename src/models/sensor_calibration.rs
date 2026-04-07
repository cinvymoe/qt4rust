// src/models/sensor_calibration.rs

use crate::algorithms::ad_converter::AdConverter;
use serde::{Deserialize, Serialize};

/// 单个传感器的标定参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorCalibrationParams {
    /// 零点 AD 值
    pub zero_ad: f64,
    /// 零点物理值
    pub zero_value: f64,
    /// 放大 AD 值（满量程）
    pub scale_ad: f64,
    /// 放大物理值
    pub scale_value: f64,
    /// 标定倍率（默认为 1.0）
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

fn default_multiplier() -> f64 {
    1.0
}

impl Default for SensorCalibrationParams {
    fn default() -> Self {
        Self {
            zero_ad: 0.0,
            zero_value: 0.0,
            scale_ad: 4095.0,
            scale_value: 50.0,
            multiplier: 1.0,
        }
    }
}

impl SensorCalibrationParams {
    /// 验证标定参数有效性
    pub fn validate(&self) -> Result<(), String> {
        AdConverter::validate_calibration(self.zero_ad, self.scale_ad)
    }
    
    /// 将 AD 值转换为物理值
    pub fn convert_ad_to_value(&self, ad: f64) -> f64 {
        AdConverter::convert(
            ad,
            self.zero_ad,
            self.zero_value,
            self.scale_ad,
            self.scale_value,
        )
    }
}

/// 传感器标定配置（结构化）
/// 
/// 存储所有传感器的标定参数，用于将 AD 采集值转换为物理值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCalibration {
    /// 重量传感器标定参数
    pub weight: SensorCalibrationParams,
    /// 角度传感器标定参数
    pub angle: SensorCalibrationParams,
    /// 半径传感器标定参数
    pub radius: SensorCalibrationParams,
}

impl Default for SensorCalibration {
    fn default() -> Self {
        Self {
            weight: SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 50.0,  // 50 tons
                multiplier: 1.0,
            },
            angle: SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 90.0,  // 90 degrees
                multiplier: 1.0,
            },
            radius: SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 20.0,  // 20 meters
                multiplier: 1.0,
            },
        }
    }
}

impl SensorCalibration {
    /// 转换重量 AD 值为物理值（吨）
    pub fn convert_weight_ad_to_value(&self, ad: f64) -> f64 {
        self.weight.convert_ad_to_value(ad)
    }
    
    /// 转换角度 AD 值为物理值（度）
    pub fn convert_angle_ad_to_value(&self, ad: f64) -> f64 {
        self.angle.convert_ad_to_value(ad)
    }
    
    /// 转换半径 AD 值为物理值（米）
    pub fn convert_radius_ad_to_value(&self, ad: f64) -> f64 {
        self.radius.convert_ad_to_value(ad)
    }
    
    /// 验证标定参数的有效性
    pub fn validate(&self) -> Result<(), String> {
        self.weight.validate()
            .map_err(|e| format!("重量传感器标定参数错误: {}", e))?;
        self.angle.validate()
            .map_err(|e| format!("角度传感器标定参数错误: {}", e))?;
        self.radius.validate()
            .map_err(|e| format!("半径传感器标定参数错误: {}", e))?;
        Ok(())
    }
}

/// 报警阈值配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlarmThresholds {
    /// 角度报警阈值
    pub angle: AngleThresholds,
    /// 力矩报警阈值
    pub moment: MomentThresholds,
}

impl Default for AlarmThresholds {
    fn default() -> Self {
        Self {
            angle: AngleThresholds::default(),
            moment: MomentThresholds::default(),
        }
    }
}

/// 角度报警阈值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AngleThresholds {
    /// 预警值（度）
    pub warning: f64,
    /// 报警值（度）
    pub alarm: f64,
}

impl Default for AngleThresholds {
    fn default() -> Self {
        Self {
            warning: 75.0,
            alarm: 85.0,
        }
    }
}

/// 力矩报警阈值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MomentThresholds {
    /// 预警百分比（%）
    pub warning_percentage: f64,
    /// 报警百分比（%）
    pub alarm_percentage: f64,
}

impl Default for MomentThresholds {
    fn default() -> Self {
        Self {
            warning_percentage: 90.0,
            alarm_percentage: 100.0,
        }
    }
}

impl AlarmThresholds {
    /// 检查角度是否超过预警值
    pub fn is_angle_warning(&self, angle: f64) -> bool {
        angle >= self.angle.warning
    }
    
    /// 检查角度是否超过报警值
    pub fn is_angle_alarm(&self, angle: f64) -> bool {
        angle >= self.angle.alarm
    }
    
    /// 检查力矩百分比是否超过预警值
    pub fn is_moment_warning(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment.warning_percentage
    }
    
    /// 检查力矩百分比是否超过报警值
    pub fn is_moment_alarm(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment.alarm_percentage
    }
    
    /// 验证报警阈值的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证角度预警值范围
        if self.angle.warning < 0.0 || self.angle.warning > 90.0 {
            return Err(format!(
                "angle.warning 必须在 0-90 范围内，当前值: {}",
                self.angle.warning
            ));
        }
        
        // 验证角度报警值范围
        if self.angle.alarm < 0.0 || self.angle.alarm > 90.0 {
            return Err(format!(
                "angle.alarm 必须在 0-90 范围内，当前值: {}",
                self.angle.alarm
            ));
        }
        
        // 验证角度报警值必须大于等于预警值
        if self.angle.alarm < self.angle.warning {
            return Err(format!(
                "angle.alarm ({}) 必须大于等于 angle.warning ({})",
                self.angle.alarm, self.angle.warning
            ));
        }
        
        // 验证力矩预警百分比范围
        if self.moment.warning_percentage < 0.0 || self.moment.warning_percentage > 100.0 {
            return Err(format!(
                "moment.warning_percentage 必须在 0-100 范围内，当前值: {}",
                self.moment.warning_percentage
            ));
        }
        
        // 验证力矩报警百分比范围
        if self.moment.alarm_percentage < 0.0 || self.moment.alarm_percentage > 100.0 {
            return Err(format!(
                "moment.alarm_percentage 必须在 0-100 范围内，当前值: {}",
                self.moment.alarm_percentage
            ));
        }
        
        // 验证力矩报警百分比必须大于等于预警百分比
        if self.moment.alarm_percentage < self.moment.warning_percentage {
            return Err(format!(
                "moment.alarm_percentage ({}) 必须大于等于 moment.warning_percentage ({})",
                self.moment.alarm_percentage, self.moment.warning_percentage
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_weight_ad_to_value() {
        let calibration = SensorCalibration::default();
        
        // 测试零点
        let weight = calibration.convert_weight_ad_to_value(0.0);
        assert!((weight - 0.0).abs() < 0.01);
        
        // 测试中点
        let weight = calibration.convert_weight_ad_to_value(2047.5);
        assert!((weight - 25.0).abs() < 0.1);
        
        // 测试满量程
        let weight = calibration.convert_weight_ad_to_value(4095.0);
        assert!((weight - 50.0).abs() < 0.01);
    }
    
    #[test]
    fn test_convert_angle_ad_to_value() {
        let calibration = SensorCalibration::default();
        
        let angle = calibration.convert_angle_ad_to_value(0.0);
        assert!((angle - 0.0).abs() < 0.01);
        
        let angle = calibration.convert_angle_ad_to_value(4095.0);
        assert!((angle - 90.0).abs() < 0.01);
    }
    
    #[test]
    fn test_convert_radius_ad_to_value() {
        let calibration = SensorCalibration::default();
        
        let radius = calibration.convert_radius_ad_to_value(0.0);
        assert!((radius - 0.0).abs() < 0.01);
        
        let radius = calibration.convert_radius_ad_to_value(4095.0);
        assert!((radius - 20.0).abs() < 0.01);
    }
    
    #[test]
    fn test_alarm_thresholds_angle_warning() {
        let thresholds = AlarmThresholds::default();
        
        assert!(!thresholds.is_angle_warning(60.0));
        assert!(!thresholds.is_angle_warning(74.9));
        assert!(thresholds.is_angle_warning(75.0));
        assert!(thresholds.is_angle_warning(80.0));
    }
    
    #[test]
    fn test_alarm_thresholds_angle_alarm() {
        let thresholds = AlarmThresholds::default();
        
        assert!(!thresholds.is_angle_alarm(60.0));
        assert!(!thresholds.is_angle_alarm(84.9));
        assert!(thresholds.is_angle_alarm(85.0));
        assert!(thresholds.is_angle_alarm(88.0));
    }
    
    #[test]
    fn test_alarm_thresholds_moment_warning() {
        let thresholds = AlarmThresholds::default();
        
        assert!(!thresholds.is_moment_warning(80.0));
        assert!(!thresholds.is_moment_warning(89.9));
        assert!(thresholds.is_moment_warning(90.0));
        assert!(thresholds.is_moment_warning(95.0));
    }
    
    #[test]
    fn test_alarm_thresholds_moment_alarm() {
        let thresholds = AlarmThresholds::default();
        
        assert!(!thresholds.is_moment_alarm(80.0));
        assert!(!thresholds.is_moment_alarm(99.9));
        assert!(thresholds.is_moment_alarm(100.0));
        assert!(thresholds.is_moment_alarm(105.0));
    }
    
    #[test]
    fn test_alarm_thresholds_validate_success() {
        let thresholds = AlarmThresholds::default();
        assert!(thresholds.validate().is_ok());
    }
    
    #[test]
    fn test_alarm_thresholds_validate_angle_range() {
        let mut thresholds = AlarmThresholds::default();
        thresholds.angle.warning = -10.0;
        assert!(thresholds.validate().is_err());
        
        thresholds.angle.warning = 95.0;
        assert!(thresholds.validate().is_err());
    }
    
    #[test]
    fn test_alarm_thresholds_validate_moment_range() {
        let mut thresholds = AlarmThresholds::default();
        thresholds.moment.warning_percentage = -10.0;
        assert!(thresholds.validate().is_err());
        
        thresholds.moment.warning_percentage = 110.0;
        assert!(thresholds.validate().is_err());
    }
    
    #[test]
    fn test_sensor_calibration_params_convert() {
        let params = SensorCalibrationParams {
            zero_ad: 100.0,
            zero_value: 5.0,
            scale_ad: 4000.0,
            scale_value: 45.0,
        };
        
        let value = params.convert_ad_to_value(100.0);
        assert!((value - 5.0).abs() < 0.01);
        
        let value = params.convert_ad_to_value(4000.0);
        assert!((value - 45.0).abs() < 0.01);
    }
}