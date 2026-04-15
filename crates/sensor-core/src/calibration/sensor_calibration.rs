use crate::algorithms::ad_converter::AdConverter;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCalibration {
    pub weight: SensorCalibrationParams,
    pub angle: SensorCalibrationParams,
    pub radius: SensorCalibrationParams,
}

impl Default for SensorCalibration {
    fn default() -> Self {
        Self {
            weight: SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 50.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
            angle: SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 90.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
            radius: SensorCalibrationParams {
                zero_ad: 0.0,
                zero_value: 0.0,
                scale_ad: 4095.0,
                scale_value: 20.0,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
        }
    }
}

impl SensorCalibration {
    pub fn convert_weight_ad_to_value(&self, ad: f64) -> f64 {
        self.weight.convert_ad_to_value(ad)
    }

    pub fn convert_angle_ad_to_value(&self, ad: f64) -> f64 {
        self.angle.convert_ad_to_value(ad)
    }

    pub fn convert_radius_ad_to_value(&self, ad: f64) -> f64 {
        self.radius.convert_ad_to_value(ad)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.weight
            .validate()
            .map_err(|e| format!("重量传感器标定参数错误: {}", e))?;
        self.angle
            .validate()
            .map_err(|e| format!("角度传感器标定参数错误: {}", e))?;
        self.radius
            .validate()
            .map_err(|e| format!("半径传感器标定参数错误: {}", e))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlarmThresholds {
    pub moment: MomentThresholds,
    #[serde(default)]
    pub angle: AngleThresholds,
    #[serde(default)]
    pub main_hook_switch: HookSwitchThresholds,
    #[serde(default)]
    pub aux_hook_switch: HookSwitchThresholds,
}

impl Default for AlarmThresholds {
    fn default() -> Self {
        Self {
            moment: MomentThresholds::default(),
            angle: AngleThresholds::default(),
            main_hook_switch: HookSwitchThresholds::default(),
            aux_hook_switch: HookSwitchThresholds::default(),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HookSwitchMode {
    None,
    NormallyOpen,
    NormallyClosed,
}

impl Default for HookSwitchMode {
    fn default() -> Self {
        Self::None
    }
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
    fn test_convert_weight_ad_to_value() {
        let calibration = SensorCalibration::default();

        let weight = calibration.convert_weight_ad_to_value(0.0);
        assert!((weight - 0.0).abs() < 0.01);

        let weight = calibration.convert_weight_ad_to_value(2047.5);
        assert!((weight - 25.0).abs() < 0.1);

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
