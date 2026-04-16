// src/pipeline/calibration_service.rs

use crate::algorithms::ad_converter::AdConverter;
use sensor_core::SensorCalibration;
use std::sync::{Arc, RwLock};
use tracing::trace;

/// AD 转换服务
///
/// 封装传感器 AD 值到物理值的转换逻辑。
/// 支持热重载：当 SensorCalibration 更新时，
/// 后续转换自动使用新参数。
pub struct CalibrationService {
    calibration: Arc<RwLock<SensorCalibration>>,
}

impl CalibrationService {
    /// 创建新的转换服务
    pub fn new(calibration: Arc<RwLock<SensorCalibration>>) -> Self {
        Self { calibration }
    }

    /// 从 ConfigProvider 创建
    pub fn from_provider(provider: &super::ConfigProvider) -> Self {
        Self {
            calibration: provider.get_sensor_calibration_arc(),
        }
    }

    /// 转换重量 AD 值到物理值（吨）
    pub fn convert_weight(&self, ad_value: f64) -> f64 {
        let cal = self.calibration.read().unwrap();
        let raw = AdConverter::convert(
            ad_value,
            cal.weight.zero_ad,
            cal.weight.zero_value,
            cal.weight.scale_ad,
            cal.weight.scale_value,
        );
        raw * cal.weight.multiplier
    }

    /// 转换角度 AD 值到物理值（度）
    pub fn convert_angle(&self, ad_value: f64) -> f64 {
        let cal = self.calibration.read().unwrap();
        AdConverter::convert(
            ad_value,
            cal.angle.zero_ad,
            cal.angle.zero_value,
            cal.angle.scale_ad,
            cal.angle.scale_value,
        )
    }

    /// 转换半径 AD 值到物理值（米）
    pub fn convert_radius(&self, ad_value: f64) -> f64 {
        let cal = self.calibration.read().unwrap();
        AdConverter::convert(
            ad_value,
            cal.radius.zero_ad,
            cal.radius.zero_value,
            cal.radius.scale_ad,
            cal.radius.scale_value,
        )
    }

    /// 批量转换传感器数据
    ///
    /// 返回 (weight, angle, radius) 元组
    pub fn convert_sensor_data(
        &self,
        weight_ad: f64,
        angle_ad: f64,
        radius_ad: f64,
    ) -> (f64, f64, f64) {
        let weight = self.convert_weight(weight_ad);
        let angle = self.convert_angle(angle_ad);
        let radius = self.convert_radius(radius_ad);

        trace!(
            "AD conversion: weight_ad={:.1} -> {:.2}t, angle_ad={:.1} -> {:.1}°, radius_ad={:.1} -> {:.2}m",
            weight_ad, weight,
            angle_ad, angle,
            radius_ad, radius
        );

        (weight, angle, radius)
    }

    /// 获取当前标定参数的快照
    pub fn get_calibration(&self) -> SensorCalibration {
        self.calibration.read().unwrap().clone()
    }

    /// 更新标定参数
    pub fn update_calibration(&self, calibration: SensorCalibration) {
        *self.calibration.write().unwrap() = calibration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensor_core::SensorCalibration;

    fn create_test_calibration() -> SensorCalibration {
        let mut cal = SensorCalibration::default();
        cal.weight.zero_ad = 0.0;
        cal.weight.zero_value = 0.0;
        cal.weight.scale_ad = 4095.0;
        cal.weight.scale_value = 50.0;
        cal.weight.multiplier = 1.0;
        cal.angle.zero_ad = 0.0;
        cal.angle.zero_value = 0.0;
        cal.angle.scale_ad = 4095.0;
        cal.angle.scale_value = 90.0;
        cal.radius.zero_ad = 0.0;
        cal.radius.zero_value = 0.0;
        cal.radius.scale_ad = 4095.0;
        cal.radius.scale_value = 20.0;
        cal
    }

    #[test]
    fn test_convert_weight() {
        let cal = create_test_calibration();
        let service = CalibrationService::new(Arc::new(RwLock::new(cal)));

        // AD 中点应该对应 25 吨
        let weight = service.convert_weight(2047.5);
        assert!((weight - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_convert_angle() {
        let cal = create_test_calibration();
        let service = CalibrationService::new(Arc::new(RwLock::new(cal)));

        // AD 满量程应该对应 90 度
        let angle = service.convert_angle(4095.0);
        assert!((angle - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_convert_radius() {
        let cal = create_test_calibration();
        let service = CalibrationService::new(Arc::new(RwLock::new(cal)));

        // AD 中点应该对应 10 米
        let radius = service.convert_radius(2047.5);
        assert!((radius - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_convert_sensor_data() {
        let cal = create_test_calibration();
        let service = CalibrationService::new(Arc::new(RwLock::new(cal)));

        let (weight, angle, radius) = service.convert_sensor_data(2047.5, 2047.5, 2047.5);
        assert!((weight - 25.0).abs() < 0.1);
        assert!((angle - 45.0).abs() < 0.1);
        assert!((radius - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_get_calibration() {
        let cal = create_test_calibration();
        let service = CalibrationService::new(Arc::new(RwLock::new(cal.clone())));

        let retrieved = service.get_calibration();
        assert_eq!(retrieved.weight.scale_value, 50.0);
        assert_eq!(retrieved.angle.scale_value, 90.0);
        assert_eq!(retrieved.radius.scale_value, 20.0);
    }

    #[test]
    fn test_update_calibration() {
        let cal = create_test_calibration();
        let service = CalibrationService::new(Arc::new(RwLock::new(cal)));

        let mut new_cal = SensorCalibration::default();
        new_cal.weight.scale_value = 100.0;

        service.update_calibration(new_cal.clone());

        let retrieved = service.get_calibration();
        assert_eq!(retrieved.weight.scale_value, 100.0);
    }

    #[test]
    fn test_from_provider() {
        let provider = super::super::ConfigProvider::new();
        let service = CalibrationService::from_provider(&provider);

        // 使用默认标定参数转换
        let weight = service.convert_weight(2047.5);
        assert!((weight - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_shared_calibration_updates() {
        let cal = create_test_calibration();
        let arc_cal = Arc::new(RwLock::new(cal));

        let service1 = CalibrationService::new(Arc::clone(&arc_cal));
        let service2 = CalibrationService::new(Arc::clone(&arc_cal));

        // 更新 via service1
        let mut new_cal = SensorCalibration::default();
        new_cal.weight.scale_value = 100.0;
        service1.update_calibration(new_cal);

        // service2 应该看到更新
        let weight = service2.convert_weight(4095.0);
        assert!((weight - 100.0).abs() < 0.1);
    }
}
