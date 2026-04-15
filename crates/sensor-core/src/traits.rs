use crate::error::SensorResult;

pub trait SensorSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64)>;
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
    data: Vec<(f64, f64, f64)>,
    current_index: std::cell::Cell<usize>,
}

#[cfg(test)]
impl MockSensorSource {
    pub fn new(data: Vec<(f64, f64, f64)>) -> Self {
        Self {
            data,
            current_index: std::cell::Cell::new(0),
        }
    }
}

#[cfg(test)]
impl SensorSource for MockSensorSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64)> {
        let index = self.current_index.get();
        if index < self.data.len() {
            let value = self.data[index];
            self.current_index.set(index + 1);
            Ok(value)
        } else {
            // Return last value when exhausted
            Ok(*self.data.last().unwrap_or(&(0.0, 0.0, 0.0)))
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
        let mock = MockSensorSource::new(vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0), (7.0, 8.0, 9.0)]);

        assert_eq!(mock.read_all().unwrap(), (1.0, 2.0, 3.0));
        assert_eq!(mock.read_all().unwrap(), (4.0, 5.0, 6.0));
        assert_eq!(mock.read_all().unwrap(), (7.0, 8.0, 9.0));
    }

    #[test]
    fn test_mock_sensor_source_returns_last_when_exhausted() {
        let mock = MockSensorSource::new(vec![(1.0, 2.0, 3.0), (4.0, 5.0, 6.0)]);

        mock.read_all().unwrap();
        mock.read_all().unwrap();

        // After exhausted, should return last value
        assert_eq!(mock.read_all().unwrap(), (4.0, 5.0, 6.0));
        assert_eq!(mock.read_all().unwrap(), (4.0, 5.0, 6.0));
    }

    #[test]
    fn test_mock_sensor_source_is_connected() {
        let mock_with_data = MockSensorSource::new(vec![(1.0, 2.0, 3.0)]);
        assert!(mock_with_data.is_connected());

        let mock_empty = MockSensorSource::new(vec![]);
        assert!(!mock_empty.is_connected());
    }
}
