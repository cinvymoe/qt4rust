use crate::error::SensorResult;

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait SensorSource {
    /// 读取所有传感器数据（3个模拟量 + 2个数字量）
    /// 返回 (weight, angle, radius, digital_input_0, digital_input_1)
    fn read_all(&self) -> SensorResult<(f64, f64, f64, bool, bool)>;
    fn is_connected(&self) -> bool;
}

pub trait SensorProvider {
    fn read(&self) -> SensorResult<f64>;
    fn is_connected(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "Unknown Sensor"
    }
}

#[cfg(test)]
pub struct MockSensorSource {
    data: Vec<(f64, f64, f64, bool, bool)>,
    current_index: AtomicUsize,
}

#[cfg(test)]
impl MockSensorSource {
    pub fn new(data: Vec<(f64, f64, f64, bool, bool)>) -> Self {
        Self {
            data,
            current_index: AtomicUsize::new(0),
        }
    }
}

#[cfg(test)]
impl SensorSource for MockSensorSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64, bool, bool)> {
        let index = self.current_index.fetch_add(1, Ordering::SeqCst);
        if index < self.data.len() {
            Ok(self.data[index])
        } else {
            Ok(*self.data.last().unwrap_or(&(0.0, 0.0, 0.0, false, false)))
        }
    }

    fn is_connected(&self) -> bool {
        !self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_sensor_source_returns_sequential_data() {
        let mock = MockSensorSource::new(vec![
            (1.0, 2.0, 3.0, false, false),
            (4.0, 5.0, 6.0, true, false),
            (7.0, 8.0, 9.0, false, true),
        ]);

        assert_eq!(mock.read_all().unwrap(), (1.0, 2.0, 3.0, false, false));
        assert_eq!(mock.read_all().unwrap(), (4.0, 5.0, 6.0, true, false));
        assert_eq!(mock.read_all().unwrap(), (7.0, 8.0, 9.0, false, true));
    }

    #[test]
    fn test_mock_sensor_source_returns_last_when_exhausted() {
        let mock = MockSensorSource::new(vec![
            (1.0, 2.0, 3.0, false, false),
            (4.0, 5.0, 6.0, true, true),
        ]);

        mock.read_all().unwrap();
        mock.read_all().unwrap();

        assert_eq!(mock.read_all().unwrap(), (4.0, 5.0, 6.0, true, true));
        assert_eq!(mock.read_all().unwrap(), (4.0, 5.0, 6.0, true, true));
    }

    #[test]
    fn test_mock_sensor_source_is_connected() {
        let mock_with_data = MockSensorSource::new(vec![(1.0, 2.0, 3.0, false, false)]);
        assert!(mock_with_data.is_connected());

        let mock_empty = MockSensorSource::new(vec![]);
        assert!(!mock_empty.is_connected());
    }
}
