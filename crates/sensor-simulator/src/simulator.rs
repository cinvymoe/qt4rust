// Sensor Simulators (Legacy API - 保持向后兼容)
//
// 推荐使用新的 SimulatedSensor API

use crate::config::SimulatorConfig;
use crate::simulated::{SimulatedSensor, SimulatorType};
use crate::traits::SensorProvider;
use std::time::SystemTime;

/// 正弦波模拟器（向后兼容）
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

    /// 生成数据（旧 API）
    pub fn generate(&self) -> f64 {
        let elapsed = self.start_time.elapsed().unwrap_or_default().as_secs_f64();
        let value = self.config.amplitude 
            * (2.0 * std::f64::consts::PI * self.config.frequency * elapsed).sin()
            + self.config.offset;
        value
    }

    /// 转换为新的 SimulatedSensor
    pub fn into_sensor(self) -> SimulatedSensor {
        SimulatedSensor::new(SimulatorType::Sine(self.config))
    }
}

impl Default for SineSimulator {
    fn default() -> Self {
        Self::new(SimulatorConfig::default())
    }
}

// 实现新的 SensorProvider trait
impl SensorProvider for SineSimulator {
    fn read(&self) -> crate::error::SensorResult<f64> {
        Ok(self.generate())
    }

    fn name(&self) -> &str {
        "Sine Simulator (Legacy)"
    }
}
