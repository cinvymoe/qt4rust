// src/models/sensor_calibration.rs

use crate::algorithms::ad_converter::AdConverter;
use serde::{Deserialize, Serialize};

/// 传感器标定配置
/// 
/// 存储所有传感器的标定参数，用于将 AD 采集值转换为物理值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCalibration {
    // ========== 重量传感器标定参数 ==========
    /// 重量零点 AD 值
    pub weight_zero_ad: f64,
    /// 重量零点物理值（吨）
    pub weight_zero_value: f64,
    /// 重量放大 AD 值
    pub weight_scale_ad: f64,
    /// 重量放大物理值（吨）
    pub weight_scale_value: f64,
    
    // ========== 角度传感器标定参数 ==========
    /// 角度零点 AD 值
    pub angle_zero_ad: f64,
    /// 角度零点物理值（度）
    pub angle_zero_value: f64,
    /// 角度放大 AD 值
    pub angle_scale_ad: f64,
    /// 角度放大物理值（度）
    pub angle_scale_value: f64,
    
    // ========== 半径传感器标定参数 ==========
    /// 半径零点 AD 值
    pub radius_zero_ad: f64,
    /// 半径零点物理值（米）
    pub radius_zero_value: f64,
    /// 半径放大 AD 值
    pub radius_scale_ad: f64,
    /// 半径放大物理值（米）
    pub radius_scale_value: f64,
    
    // ========== 预警值和报警值 ==========
    /// 角度预警值（度）
    pub angle_warning_value: f64,
    /// 角度报警值（度）
    pub angle_alarm_value: f64,
    /// 力矩预警百分比（%）
    pub moment_warning_percentage: f64,
    /// 力矩报警百分比（%）
    pub moment_alarm_percentage: f64,
}

impl SensorCalibration {
    /// 转换重量 AD 值为物理值（吨）
    /// 
    /// # 参数
    /// - `ad`: 重量传感器 AD 采集值
    /// 
    /// # 返回
    /// 转换后的重量物理值（吨）
    pub fn convert_weight_ad_to_value(&self, ad: f64) -> f64 {
        AdConverter::convert(
            ad,
            self.weight_zero_ad,
            self.weight_zero_value,
            self.weight_scale_ad,
            self.weight_scale_value,
        )
    }
    
    /// 转换角度 AD 值为物理值（度）
    /// 
    /// # 参数
    /// - `ad`: 角度传感器 AD 采集值
    /// 
    /// # 返回
    /// 转换后的角度物理值（度）
    pub fn convert_angle_ad_to_value(&self, ad: f64) -> f64 {
        AdConverter::convert(
            ad,
            self.angle_zero_ad,
            self.angle_zero_value,
            self.angle_scale_ad,
            self.angle_scale_value,
        )
    }
    
    /// 转换半径 AD 值为物理值（米）
    /// 
    /// # 参数
    /// - `ad`: 半径传感器 AD 采集值
    /// 
    /// # 返回
    /// 转换后的半径物理值（米）
    pub fn convert_radius_ad_to_value(&self, ad: f64) -> f64 {
        AdConverter::convert(
            ad,
            self.radius_zero_ad,
            self.radius_zero_value,
            self.radius_scale_ad,
            self.radius_scale_value,
        )
    }
    
    /// 检查角度是否超过预警值
    /// 
    /// # 参数
    /// - `angle`: 当前角度值（度）
    /// 
    /// # 返回
    /// 如果角度 >= 预警值，返回 true
    pub fn is_angle_warning(&self, angle: f64) -> bool {
        angle >= self.angle_warning_value
    }
    
    /// 检查角度是否超过报警值
    /// 
    /// # 参数
    /// - `angle`: 当前角度值（度）
    /// 
    /// # 返回
    /// 如果角度 >= 报警值，返回 true
    pub fn is_angle_alarm(&self, angle: f64) -> bool {
        angle >= self.angle_alarm_value
    }
    
    /// 检查力矩百分比是否超过预警值
    /// 
    /// # 参数
    /// - `moment_percentage`: 当前力矩百分比（%）
    /// 
    /// # 返回
    /// 如果力矩百分比 >= 预警值，返回 true
    pub fn is_moment_warning(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment_warning_percentage
    }
    
    /// 检查力矩百分比是否超过报警值
    /// 
    /// # 参数
    /// - `moment_percentage`: 当前力矩百分比（%）
    /// 
    /// # 返回
    /// 如果力矩百分比 >= 报警值，返回 true
    pub fn is_moment_alarm(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment_alarm_percentage
    }
    
    /// 验证标定参数的有效性
    /// 
    /// 检查所有标定参数和阈值是否有效，包括：
    /// - 标定参数不能导致除零错误
    /// - 角度预警值和报警值必须在 0-90 范围内
    /// - 报警值必须大于等于预警值
    /// - 力矩百分比阈值必须在 0-100 范围内
    /// 
    /// # 返回
    /// - `Ok(())`: 所有参数有效
    /// - `Err(String)`: 参数无效，包含错误描述
    pub fn validate(&self) -> Result<(), String> {
        // 验证重量传感器标定参数（避免除零）
        AdConverter::validate_calibration(self.weight_zero_ad, self.weight_scale_ad)
            .map_err(|e| format!("重量传感器标定参数错误: {}", e))?;
        
        // 验证角度传感器标定参数（避免除零）
        AdConverter::validate_calibration(self.angle_zero_ad, self.angle_scale_ad)
            .map_err(|e| format!("角度传感器标定参数错误: {}", e))?;
        
        // 验证半径传感器标定参数（避免除零）
        AdConverter::validate_calibration(self.radius_zero_ad, self.radius_scale_ad)
            .map_err(|e| format!("半径传感器标定参数错误: {}", e))?;
        
        // 验证角度预警值范围
        if self.angle_warning_value < 0.0 || self.angle_warning_value > 90.0 {
            return Err(format!(
                "angle_warning_value 必须在 0-90 范围内，当前值: {}",
                self.angle_warning_value
            ));
        }
        
        // 验证角度报警值范围
        if self.angle_alarm_value < 0.0 || self.angle_alarm_value > 90.0 {
            return Err(format!(
                "angle_alarm_value 必须在 0-90 范围内，当前值: {}",
                self.angle_alarm_value
            ));
        }
        
        // 验证角度报警值必须大于等于预警值
        if self.angle_alarm_value < self.angle_warning_value {
            return Err(format!(
                "angle_alarm_value ({}) 必须大于等于 angle_warning_value ({})",
                self.angle_alarm_value, self.angle_warning_value
            ));
        }
        
        // 验证力矩预警百分比范围
        if self.moment_warning_percentage < 0.0 || self.moment_warning_percentage > 100.0 {
            return Err(format!(
                "moment_warning_percentage 必须在 0-100 范围内，当前值: {}",
                self.moment_warning_percentage
            ));
        }
        
        // 验证力矩报警百分比范围
        if self.moment_alarm_percentage < 0.0 || self.moment_alarm_percentage > 100.0 {
            return Err(format!(
                "moment_alarm_percentage 必须在 0-100 范围内，当前值: {}",
                self.moment_alarm_percentage
            ));
        }
        
        // 验证力矩报警百分比必须大于等于预警百分比
        if self.moment_alarm_percentage < self.moment_warning_percentage {
            return Err(format!(
                "moment_alarm_percentage ({}) 必须大于等于 moment_warning_percentage ({})",
                self.moment_alarm_percentage, self.moment_warning_percentage
            ));
        }
        
        Ok(())
    }
}

impl Default for SensorCalibration {
    /// 提供合理的默认标定值
    /// 
    /// 默认配置：
    /// - 重量传感器: 0-4095 AD -> 0-50 吨
    /// - 角度传感器: 0-4095 AD -> 0-90 度
    /// - 半径传感器: 0-4095 AD -> 0-20 米
    /// - 角度预警: 75 度
    /// - 角度报警: 85 度
    /// - 力矩预警: 90%
    /// - 力矩报警: 100%
    fn default() -> Self {
        Self {
            // 重量传感器标定（12位 AD，0-50 吨）
            weight_zero_ad: 0.0,
            weight_zero_value: 0.0,
            weight_scale_ad: 4095.0,
            weight_scale_value: 50.0,
            
            // 角度传感器标定（12位 AD，0-90 度）
            angle_zero_ad: 0.0,
            angle_zero_value: 0.0,
            angle_scale_ad: 4095.0,
            angle_scale_value: 90.0,
            
            // 半径传感器标定（12位 AD，0-20 米）
            radius_zero_ad: 0.0,
            radius_zero_value: 0.0,
            radius_scale_ad: 4095.0,
            radius_scale_value: 20.0,
            
            // 默认预警值和报警值
            angle_warning_value: 75.0,           // 75 度预警
            angle_alarm_value: 85.0,             // 85 度报警
            moment_warning_percentage: 90.0,     // 90% 力矩预警
            moment_alarm_percentage: 100.0,      // 100% 力矩报警
        }
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
        
        // 测试零点
        let angle = calibration.convert_angle_ad_to_value(0.0);
        assert!((angle - 0.0).abs() < 0.01);
        
        // 测试中点
        let angle = calibration.convert_angle_ad_to_value(2047.5);
        assert!((angle - 45.0).abs() < 0.1);
        
        // 测试满量程
        let angle = calibration.convert_angle_ad_to_value(4095.0);
        assert!((angle - 90.0).abs() < 0.01);
    }
    
    #[test]
    fn test_convert_radius_ad_to_value() {
        let calibration = SensorCalibration::default();
        
        // 测试零点
        let radius = calibration.convert_radius_ad_to_value(0.0);
        assert!((radius - 0.0).abs() < 0.01);
        
        // 测试中点
        let radius = calibration.convert_radius_ad_to_value(2047.5);
        assert!((radius - 10.0).abs() < 0.1);
        
        // 测试满量程
        let radius = calibration.convert_radius_ad_to_value(4095.0);
        assert!((radius - 20.0).abs() < 0.01);
    }
    
    #[test]
    fn test_is_angle_warning() {
        let calibration = SensorCalibration::default();
        
        // 低于预警值
        assert!(!calibration.is_angle_warning(60.0));
        assert!(!calibration.is_angle_warning(74.9));
        
        // 等于预警值
        assert!(calibration.is_angle_warning(75.0));
        
        // 高于预警值
        assert!(calibration.is_angle_warning(80.0));
        assert!(calibration.is_angle_warning(85.0));
    }
    
    #[test]
    fn test_is_angle_alarm() {
        let calibration = SensorCalibration::default();
        
        // 低于报警值
        assert!(!calibration.is_angle_alarm(60.0));
        assert!(!calibration.is_angle_alarm(84.9));
        
        // 等于报警值
        assert!(calibration.is_angle_alarm(85.0));
        
        // 高于报警值
        assert!(calibration.is_angle_alarm(88.0));
        assert!(calibration.is_angle_alarm(90.0));
    }
    
    #[test]
    fn test_is_moment_warning() {
        let calibration = SensorCalibration::default();
        
        // 低于预警值
        assert!(!calibration.is_moment_warning(80.0));
        assert!(!calibration.is_moment_warning(89.9));
        
        // 等于预警值
        assert!(calibration.is_moment_warning(90.0));
        
        // 高于预警值
        assert!(calibration.is_moment_warning(95.0));
        assert!(calibration.is_moment_warning(100.0));
    }
    
    #[test]
    fn test_is_moment_alarm() {
        let calibration = SensorCalibration::default();
        
        // 低于报警值
        assert!(!calibration.is_moment_alarm(80.0));
        assert!(!calibration.is_moment_alarm(99.9));
        
        // 等于报警值
        assert!(calibration.is_moment_alarm(100.0));
        
        // 高于报警值
        assert!(calibration.is_moment_alarm(105.0));
        assert!(calibration.is_moment_alarm(110.0));
    }
    
    #[test]
    fn test_validate_success() {
        let calibration = SensorCalibration::default();
        assert!(calibration.validate().is_ok());
    }
    
    #[test]
    fn test_validate_weight_calibration_error() {
        let mut calibration = SensorCalibration::default();
        calibration.weight_scale_ad = calibration.weight_zero_ad;
        
        let result = calibration.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("重量传感器"));
    }
    
    #[test]
    fn test_validate_angle_calibration_error() {
        let mut calibration = SensorCalibration::default();
        calibration.angle_scale_ad = calibration.angle_zero_ad;
        
        let result = calibration.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("角度传感器"));
    }
    
    #[test]
    fn test_validate_radius_calibration_error() {
        let mut calibration = SensorCalibration::default();
        calibration.radius_scale_ad = calibration.radius_zero_ad;
        
        let result = calibration.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("半径传感器"));
    }
    
    #[test]
    fn test_validate_angle_warning_range() {
        let mut calibration = SensorCalibration::default();
        
        // 负值
        calibration.angle_warning_value = -10.0;
        assert!(calibration.validate().is_err());
        
        // 超过 90 度
        calibration.angle_warning_value = 95.0;
        assert!(calibration.validate().is_err());
        
        // 边界值有效
        calibration.angle_warning_value = 0.0;
        calibration.angle_alarm_value = 90.0;
        assert!(calibration.validate().is_ok());
    }
    
    #[test]
    fn test_validate_angle_alarm_range() {
        let mut calibration = SensorCalibration::default();
        
        // 负值
        calibration.angle_alarm_value = -10.0;
        assert!(calibration.validate().is_err());
        
        // 超过 90 度
        calibration.angle_alarm_value = 95.0;
        assert!(calibration.validate().is_err());
    }
    
    #[test]
    fn test_validate_angle_alarm_less_than_warning() {
        let mut calibration = SensorCalibration::default();
        calibration.angle_alarm_value = 70.0;
        calibration.angle_warning_value = 75.0;
        
        let result = calibration.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("angle_alarm_value"));
        assert!(error_msg.contains("angle_warning_value"));
    }
    
    #[test]
    fn test_validate_angle_alarm_equal_warning() {
        let mut calibration = SensorCalibration::default();
        calibration.angle_alarm_value = 75.0;
        calibration.angle_warning_value = 75.0;
        
        assert!(calibration.validate().is_ok());
    }
    
    #[test]
    fn test_validate_moment_warning_range() {
        let mut calibration = SensorCalibration::default();
        
        // 负值
        calibration.moment_warning_percentage = -10.0;
        assert!(calibration.validate().is_err());
        
        // 超过 100%
        calibration.moment_warning_percentage = 110.0;
        assert!(calibration.validate().is_err());
        
        // 边界值有效
        calibration.moment_warning_percentage = 0.0;
        calibration.moment_alarm_percentage = 100.0;
        assert!(calibration.validate().is_ok());
    }
    
    #[test]
    fn test_validate_moment_alarm_range() {
        let mut calibration = SensorCalibration::default();
        
        // 负值
        calibration.moment_alarm_percentage = -10.0;
        assert!(calibration.validate().is_err());
        
        // 超过 100%
        calibration.moment_alarm_percentage = 110.0;
        assert!(calibration.validate().is_err());
    }
    
    #[test]
    fn test_validate_moment_alarm_less_than_warning() {
        let mut calibration = SensorCalibration::default();
        calibration.moment_alarm_percentage = 80.0;
        calibration.moment_warning_percentage = 90.0;
        
        let result = calibration.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("moment_alarm_percentage"));
        assert!(error_msg.contains("moment_warning_percentage"));
    }
    
    #[test]
    fn test_validate_moment_alarm_equal_warning() {
        let mut calibration = SensorCalibration::default();
        calibration.moment_alarm_percentage = 90.0;
        calibration.moment_warning_percentage = 90.0;
        
        assert!(calibration.validate().is_ok());
    }
    
    #[test]
    fn test_convert_with_offset() {
        // 测试带偏移的标定参数
        let calibration = SensorCalibration {
            weight_zero_ad: 100.0,
            weight_zero_value: 5.0,
            weight_scale_ad: 4000.0,
            weight_scale_value: 45.0,
            ..Default::default()
        };
        
        // 零点
        let weight = calibration.convert_weight_ad_to_value(100.0);
        assert!((weight - 5.0).abs() < 0.01);
        
        // 满量程
        let weight = calibration.convert_weight_ad_to_value(4000.0);
        assert!((weight - 45.0).abs() < 0.01);
        
        // 中点
        let weight = calibration.convert_weight_ad_to_value(2050.0);
        let expected = 5.0 + (2050.0 - 100.0) * (45.0 - 5.0) / (4000.0 - 100.0);
        assert!((weight - expected).abs() < 0.01);
    }
}
