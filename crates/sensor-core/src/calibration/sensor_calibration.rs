use crate::algorithms::ad_converter::AdConverter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorCalibrationParams {
    pub zero_ad: f64,
    pub zero_value: f64,
    pub scale_ad: f64,
    pub scale_value: f64,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    #[serde(default = "default_multiplier")]
    pub actual_multiplier: f64,
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
            actual_multiplier: 1.0,
        }
    }
}

impl SensorCalibrationParams {
    pub fn validate(&self) -> Result<(), String> {
        AdConverter::validate_calibration(self.zero_ad, self.scale_ad)
    }

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

/// 传感器标定配置 - 使用 HashMap 存储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCalibration {
    /// 标定参数集合 (key = 传感器ID)
    #[serde(flatten)]
    pub params: HashMap<String, SensorCalibrationParams>,
}

impl Default for SensorCalibration {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl SensorCalibration {
    /// 创建空的标定配置
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// 获取指定传感器的标定参数
    pub fn get_calibration(&self, sensor_id: &str) -> Option<&SensorCalibrationParams> {
        self.params.get(sensor_id)
    }

    /// 设置传感器的标定参数
    pub fn set_calibration(&mut self, sensor_id: &str, params: SensorCalibrationParams) {
        self.params.insert(sensor_id.to_string(), params);
    }

    /// 转换 AD 值为物理值
    pub fn convert(&self, sensor_id: &str, ad: f64) -> f64 {
        self.params
            .get(sensor_id)
            .map(|p| p.convert_ad_to_value(ad))
            .unwrap_or(ad)
    }

    /// 验证所有标定参数
    pub fn validate(&self) -> Result<(), String> {
        for (id, params) in &self.params {
            params
                .validate()
                .map_err(|e| format!("传感器 {} 标定参数错误: {}", id, e))?;
        }
        Ok(())
    }

    // ===== 兼容旧 API =====

    /// 兼容旧 API: 获取主钩重量标定
    pub fn weight(&self) -> SensorCalibrationParams {
        self.params
            .get("main_hook_weight")
            .or_else(|| self.params.get("weight"))
            .cloned()
            .unwrap_or_default()
    }

    /// 兼容旧 API: 获取角度标定
    pub fn angle(&self) -> SensorCalibrationParams {
        self.params.get("angle").cloned().unwrap_or_default()
    }

    /// 兼容旧 API: 获取半径标定
    pub fn radius(&self) -> SensorCalibrationParams {
        self.params.get("radius").cloned().unwrap_or_default()
    }

    /// 兼容旧 API: 转换主钩重量
    pub fn convert_weight_ad_to_value(&self, ad: f64) -> f64 {
        if let Some(params) = self.params.get("main_hook_weight") {
            return params.convert_ad_to_value(ad);
        }
        if let Some(params) = self.params.get("weight") {
            return params.convert_ad_to_value(ad);
        }
        ad
    }

    /// 兼容旧 API: 转换角度
    pub fn convert_angle_ad_to_value(&self, ad: f64) -> f64 {
        self.convert("angle", ad)
    }

    /// 兼容旧 API: 转换半径
    pub fn convert_radius_ad_to_value(&self, ad: f64) -> f64 {
        self.convert("radius", ad)
    }

    /// 创建默认标定配置（包含常用传感器）
    pub fn with_defaults() -> Self {
        let mut params = HashMap::new();

        params.insert(
            "main_hook_weight".to_string(),
            SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 50.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
        );

        params.insert(
            "angle".to_string(),
            SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 90.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
        );

        params.insert(
            "radius".to_string(),
            SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 20.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
        );

        Self { params }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AlarmThresholds {
    #[serde(default)]
    pub moment: MomentThresholds,
    #[serde(default)]
    pub angle: AngleThresholds,
    #[serde(default)]
    pub main_hook_switch: HookSwitchThresholds,
    #[serde(default)]
    pub aux_hook_switch: HookSwitchThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MomentThresholds {
    pub warning_percentage: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AngleThresholds {
    pub min_angle: f64,
    pub max_angle: f64,
}

impl Default for AngleThresholds {
    fn default() -> Self {
        Self {
            min_angle: 0.0,
            max_angle: 85.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HookSwitchMode {
    #[default]
    None,
    NormallyOpen,
    NormallyClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HookSwitchThresholds {
    #[serde(default)]
    pub mode: HookSwitchMode,
}

impl HookSwitchThresholds {
    pub fn is_alarm_triggered(&self, state: bool) -> bool {
        match self.mode {
            HookSwitchMode::None => false,
            HookSwitchMode::NormallyOpen => state,
            HookSwitchMode::NormallyClosed => !state,
        }
    }
}

impl AlarmThresholds {
    pub fn is_moment_warning(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment.warning_percentage
    }

    pub fn is_moment_alarm(&self, moment_percentage: f64) -> bool {
        moment_percentage >= self.moment.alarm_percentage
    }

    pub fn is_angle_alarm(&self, angle: f64) -> bool {
        angle < self.angle.min_angle || angle > self.angle.max_angle
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.moment.warning_percentage < 0.0 || self.moment.warning_percentage > 100.0 {
            return Err(format!(
                "moment.warning_percentage 必须在 0-100 范围内，当前值: {}",
                self.moment.warning_percentage
            ));
        }

        if self.moment.alarm_percentage < 0.0 || self.moment.alarm_percentage > 100.0 {
            return Err(format!(
                "moment.alarm_percentage 必须在 0-100 范围内，当前值: {}",
                self.moment.alarm_percentage
            ));
        }

        if self.moment.alarm_percentage < self.moment.warning_percentage {
            return Err(format!(
                "moment.alarm_percentage ({}) 必须大于等于 moment.warning_percentage ({})",
                self.moment.alarm_percentage, self.moment.warning_percentage
            ));
        }

        if self.angle.min_angle < 0.0 || self.angle.min_angle > 90.0 {
            return Err(format!(
                "angle.min_angle 必须在 0-90 范围内，当前值: {}",
                self.angle.min_angle
            ));
        }

        if self.angle.max_angle < 0.0 || self.angle.max_angle > 90.0 {
            return Err(format!(
                "angle.max_angle 必须在 0-90 范围内，当前值: {}",
                self.angle.max_angle
            ));
        }

        if self.angle.max_angle <= self.angle.min_angle {
            return Err(format!(
                "angle.max_angle ({}) 必须大于 angle.min_angle ({})",
                self.angle.max_angle, self.angle.min_angle
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_with_calibration() {
        let mut calibration = SensorCalibration::new();
        calibration.set_calibration(
            "test_sensor",
            SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 100.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
        );

        let value = calibration.convert("test_sensor", 2047.5);
        assert!((value - 50.0).abs() < 0.5);
    }

    #[test]
    fn test_convert_missing_calibration() {
        let calibration = SensorCalibration::new();

        let value = calibration.convert("nonexistent", 100.0);
        assert_eq!(value, 100.0);
    }

    #[test]
    fn test_compatibility_api() {
        let calibration = SensorCalibration::with_defaults();

        let weight = calibration.convert_weight_ad_to_value(2047.5);
        assert!((weight - 25.0).abs() < 0.5);

        let angle = calibration.convert_angle_ad_to_value(4095.0);
        assert!((angle - 90.0).abs() < 0.01);

        let radius = calibration.convert_radius_ad_to_value(4095.0);
        assert!((radius - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_aux_hook_calibration() {
        let mut calibration = SensorCalibration::with_defaults();

        calibration.set_calibration(
            "aux_hook_weight",
            SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 25.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
        );

        let aux_weight = calibration.convert("aux_hook_weight", 4095.0);
        assert!((aux_weight - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_weight_ad_to_value() {
        let calibration = SensorCalibration::with_defaults();

        let weight = calibration.convert_weight_ad_to_value(0.0);
        assert!((weight - 0.0).abs() < 0.01);

        let weight = calibration.convert_weight_ad_to_value(2047.5);
        assert!((weight - 25.0).abs() < 0.1);

        let weight = calibration.convert_weight_ad_to_value(4095.0);
        assert!((weight - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_angle_ad_to_value() {
        let calibration = SensorCalibration::with_defaults();

        let angle = calibration.convert_angle_ad_to_value(0.0);
        assert!((angle - 0.0).abs() < 0.01);

        let angle = calibration.convert_angle_ad_to_value(4095.0);
        assert!((angle - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_radius_ad_to_value() {
        let calibration = SensorCalibration::with_defaults();

        let radius = calibration.convert_radius_ad_to_value(0.0);
        assert!((radius - 0.0).abs() < 0.01);

        let radius = calibration.convert_radius_ad_to_value(4095.0);
        assert!((radius - 20.0).abs() < 0.01);
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
            multiplier: 1.0,
            actual_multiplier: 1.0,
        };

        let value = params.convert_ad_to_value(100.0);
        assert!((value - 5.0).abs() < 0.01);

        let value = params.convert_ad_to_value(4000.0);
        assert!((value - 45.0).abs() < 0.01);
    }
}
