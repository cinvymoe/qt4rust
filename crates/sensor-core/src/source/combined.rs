use crate::{AnalogSource, DigitalInputSource, SensorResult, SensorSource};

/// 组合模拟量 + 数字输入源
pub struct CombinedSensorSource {
    analog: Box<dyn AnalogSource>,
    digital: Box<dyn DigitalInputSource>,
}

impl CombinedSensorSource {
    pub fn new(analog: Box<dyn AnalogSource>, digital: Box<dyn DigitalInputSource>) -> Self {
        Self { analog, digital }
    }
}

impl SensorSource for CombinedSensorSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64, bool, bool)> {
        let (ad1, ad2, ad3) = self.analog.read()?;
        let (di0, di1) = self.digital.read()?;
        Ok((ad1, ad2, ad3, di0, di1))
    }

    fn is_connected(&self) -> bool {
        self.analog.is_connected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAnalog {
        values: (f64, f64, f64),
        connected: bool,
    }

    impl AnalogSource for MockAnalog {
        fn read(&self) -> SensorResult<(f64, f64, f64)> {
            Ok(self.values)
        }
        fn is_connected(&self) -> bool {
            self.connected
        }
        fn source_name(&self) -> &str {
            "MockAnalog"
        }
    }

    struct MockDigital {
        values: (bool, bool),
    }

    impl DigitalInputSource for MockDigital {
        fn read(&self) -> SensorResult<(bool, bool)> {
            Ok(self.values)
        }
        fn source_name(&self) -> &str {
            "MockDigital"
        }
    }

    #[test]
    fn test_combined_read_all() {
        let analog = Box::new(MockAnalog {
            values: (1.0, 2.0, 3.0),
            connected: true,
        });
        let digital = Box::new(MockDigital {
            values: (true, false),
        });

        let combined = CombinedSensorSource::new(analog, digital);
        let result = combined.read_all().unwrap();

        assert_eq!(result, (1.0, 2.0, 3.0, true, false));
    }

    #[test]
    fn test_is_connected() {
        let analog = Box::new(MockAnalog {
            values: (0.0, 0.0, 0.0),
            connected: false,
        });
        let digital = Box::new(MockDigital {
            values: (false, false),
        });

        let combined = CombinedSensorSource::new(analog, digital);
        assert!(!combined.is_connected());
    }
}
