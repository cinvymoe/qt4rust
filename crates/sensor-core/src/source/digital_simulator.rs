use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::digital::{DigitalInputConfig, DigitalInputSourceFactory};
use crate::{DigitalInputSource, SensorResult};

/// 数字输入模拟器（每 N 秒切换状态）
pub struct SimulatedDigitalInput {
    di0: AtomicBool,
    di1: AtomicBool,
    last_toggle: Mutex<Instant>,
    toggle_interval: Duration,
}

impl SimulatedDigitalInput {
    pub fn new(toggle_interval_secs: u64) -> Self {
        Self {
            di0: AtomicBool::new(false),
            di1: AtomicBool::new(false),
            last_toggle: Mutex::new(Instant::now()),
            toggle_interval: Duration::from_secs(toggle_interval_secs),
        }
    }
}

impl DigitalInputSource for SimulatedDigitalInput {
    fn read(&self) -> SensorResult<(bool, bool)> {
        let now = Instant::now();
        let should_toggle = {
            let mut last = self.last_toggle.lock().unwrap();
            if now.duration_since(*last) >= self.toggle_interval {
                *last = now;
                true
            } else {
                false
            }
        };

        if should_toggle {
            self.di0.fetch_xor(true, Ordering::SeqCst);
            self.di1.fetch_xor(true, Ordering::SeqCst);
        }

        Ok((
            self.di0.load(Ordering::SeqCst),
            self.di1.load(Ordering::SeqCst),
        ))
    }

    fn source_name(&self) -> &str {
        "SimulatedDigitalInput"
    }
}

/// 模拟器工厂
pub struct SimulatedDigitalInputFactory;

impl DigitalInputSourceFactory for SimulatedDigitalInputFactory {
    fn create(&self, config: &DigitalInputConfig) -> SensorResult<Box<dyn DigitalInputSource>> {
        Ok(Box::new(SimulatedDigitalInput::new(
            config.toggle_interval_secs,
        )))
    }

    fn name(&self) -> &str {
        "simulator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sim = SimulatedDigitalInput::new(10);
        let (di0, di1) = sim.read().unwrap();
        assert_eq!((di0, di1), (false, false));
    }

    #[test]
    fn test_factory_creates_instance() {
        let factory = SimulatedDigitalInputFactory;
        let config = DigitalInputConfig {
            toggle_interval_secs: 5,
            ..Default::default()
        };

        let source = factory.create(&config).unwrap();
        assert_eq!(source.source_name(), "SimulatedDigitalInput");
    }

    #[test]
    fn test_factory_name() {
        let factory = SimulatedDigitalInputFactory;
        assert_eq!(factory.name(), "simulator");
    }
}
