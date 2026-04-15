use rand::Rng;

use crate::{AnalogSource, SensorResult};

/// 模拟量模拟器（随机生成 AD 值）
pub struct SimulatedAnalogSource {
    min: f64,
    max: f64,
}

impl SimulatedAnalogSource {
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 4095.0, // 12-bit ADC
        }
    }

    pub fn with_range(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}

impl Default for SimulatedAnalogSource {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalogSource for SimulatedAnalogSource {
    fn read(&self) -> SensorResult<(f64, f64, f64)> {
        let mut rng = rand::thread_rng();
        let ad1 = rng.gen_range(self.min..=self.max);
        let ad2 = rng.gen_range(self.min..=self.max);
        let ad3 = rng.gen_range(self.min..=self.max);
        Ok((ad1, ad2, ad3))
    }

    fn is_connected(&self) -> bool {
        true
    }

    fn source_name(&self) -> &str {
        "SimulatedAnalogSource"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_returns_values_in_range() {
        let source = SimulatedAnalogSource::with_range(0.0, 100.0);
        let (ad1, ad2, ad3) = source.read().unwrap();

        assert!(ad1 >= 0.0 && ad1 <= 100.0);
        assert!(ad2 >= 0.0 && ad2 <= 100.0);
        assert!(ad3 >= 0.0 && ad3 <= 100.0);
    }

    #[test]
    fn test_is_connected() {
        let source = SimulatedAnalogSource::new();
        assert!(source.is_connected());
    }

    #[test]
    fn test_source_name() {
        let source = SimulatedAnalogSource::new();
        assert_eq!(source.source_name(), "SimulatedAnalogSource");
    }
}
