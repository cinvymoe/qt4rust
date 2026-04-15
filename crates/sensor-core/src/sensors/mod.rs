mod angle;
mod base;
mod load;
mod radius;

pub use angle::AngleSensor;
pub use base::CalibratedSensor;
pub use load::LoadSensor;
pub use radius::RadiusSensor;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SensorCalibrationParams;
    use crate::{SensorProvider, SensorResult};

    struct MockProvider {
        value: f64,
    }

    impl SensorProvider for MockProvider {
        fn read(&self) -> SensorResult<f64> {
            Ok(self.value)
        }
    }

    fn test_calibration() -> SensorCalibrationParams {
        SensorCalibrationParams {
            zero_ad: 0.0,
            zero_value: 0.0,
            scale_ad: 4095.0,
            scale_value: 50.0,
            multiplier: 1.0,
            actual_multiplier: 1.0,
        }
    }

    #[test]
    fn test_calibrated_sensor_converts_ad_to_physical() {
        let provider = MockProvider { value: 2048.0 };
        let sensor = CalibratedSensor::new(test_calibration(), Box::new(provider));
        let result = sensor.read_calibrated().unwrap();
        assert!((result - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_load_sensor_creates_and_reads() {
        let provider = MockProvider { value: 2048.0 };
        let sensor = LoadSensor::new(test_calibration(), Box::new(provider));
        let result = sensor.read_tons().unwrap();
        assert!((result - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_radius_sensor_creates_and_reads() {
        let cal = SensorCalibrationParams {
            zero_ad: 0.0,
            zero_value: 0.0,
            scale_ad: 4095.0,
            scale_value: 20.0,
            multiplier: 1.0,
            actual_multiplier: 1.0,
        };
        let provider = MockProvider { value: 2048.0 };
        let sensor = RadiusSensor::new(cal, Box::new(provider));
        let result = sensor.read_meters().unwrap();
        assert!((result - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_angle_sensor_creates_and_reads() {
        let cal = SensorCalibrationParams {
            zero_ad: 0.0,
            zero_value: 0.0,
            scale_ad: 4095.0,
            scale_value: 90.0,
            multiplier: 1.0,
            actual_multiplier: 1.0,
        };
        let provider = MockProvider { value: 2048.0 };
        let sensor = AngleSensor::new(cal, Box::new(provider));
        let result = sensor.read_degrees().unwrap();
        assert!((result - 45.0).abs() < 0.1);
    }
}
