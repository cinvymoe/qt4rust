// Sensor Simulators

use crate::config::SimulatorConfig;
use std::time::SystemTime;

pub struct SineSimulator {
    config: SimulatorConfig,
    start_time: SystemTime,
}

impl SineSimulator {
    pub fn new(config: SimulatorConfig) -> Self {
        Self {
            config,
            start_time: SystemTime::now(),
        }
    }

    pub fn generate(&self) -> f64 {
        let elapsed = self.start_time.elapsed().unwrap_or_default().as_secs_f64();
        let value = self.config.amplitude 
            * (2.0 * std::f64::consts::PI * self.config.frequency * elapsed).sin()
            + self.config.offset;
        value
    }
}

impl Default for SineSimulator {
    fn default() -> Self {
        Self::new(SimulatorConfig::default())
    }
}
