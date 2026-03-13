// Simulator Configuration

#[derive(Debug, Clone)]
pub struct SimulatorConfig {
    pub amplitude: f64,
    pub frequency: f64,
    pub offset: f64,
    pub noise_level: f64,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            amplitude: 5.0,
            frequency: 0.5,
            offset: 15.0,
            noise_level: 0.1,
        }
    }
}
