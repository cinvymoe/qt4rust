use crate::config::SimulatorConfig;
use sensor_core::{SensorProvider, SensorResult};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum SimulatorType {
    Sine(SimulatorConfig),
    Random { min: f64, max: f64 },
    Constant(f64),
}

pub struct SimulatedSensor {
    simulator_type: SimulatorType,
    start_time: SystemTime,
}

impl SimulatedSensor {
    pub fn new(simulator_type: SimulatorType) -> Self {
        Self {
            simulator_type,
            start_time: SystemTime::now(),
        }
    }

    fn generate_sine(&self, config: &SimulatorConfig) -> f64 {
        let elapsed = self.start_time.elapsed().unwrap_or_default().as_secs_f64();

        let base_value = config.amplitude
            * (2.0 * std::f64::consts::PI * config.frequency * elapsed).sin()
            + config.offset;

        if config.noise_level > 0.0 {
            let noise = (rand::random::<f64>() - 0.5) * 2.0 * config.noise_level;
            base_value + noise
        } else {
            base_value
        }
    }

    fn generate_random(&self, min: f64, max: f64) -> f64 {
        min + rand::random::<f64>() * (max - min)
    }
}

impl Default for SimulatedSensor {
    fn default() -> Self {
        Self::new(SimulatorType::Sine(SimulatorConfig::default()))
    }
}

impl SensorProvider for SimulatedSensor {
    fn read(&self) -> SensorResult<f64> {
        let value = match &self.simulator_type {
            SimulatorType::Sine(config) => self.generate_sine(config),
            SimulatorType::Random { min, max } => self.generate_random(*min, *max),
            SimulatorType::Constant(value) => *value,
        };
        Ok(value)
    }

    fn name(&self) -> &str {
        match &self.simulator_type {
            SimulatorType::Sine(_) => "Sine Simulator",
            SimulatorType::Random { .. } => "Random Simulator",
            SimulatorType::Constant(_) => "Constant Simulator",
        }
    }
}

mod rand {
    use std::cell::Cell;
    use std::time::SystemTime;

    thread_local! {
        static SEED: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        );
    }

    pub fn random<T: Random>() -> T {
        T::random()
    }

    pub trait Random {
        fn random() -> Self;
    }

    impl Random for f64 {
        fn random() -> Self {
            SEED.with(|seed| {
                let mut s = seed.get();
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                seed.set(s);
                (s as f64) / (u64::MAX as f64)
            })
        }
    }
}
