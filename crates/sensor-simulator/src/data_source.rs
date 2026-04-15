use crate::simulated::{SimulatedSensor, SimulatorType};
use sensor_core::{SensorProvider, SensorResult, SensorSource};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct SimulatedDataSource {
    // 模拟传感器（高频采样）
    ad1: SimulatedSensor,
    ad2: SimulatedSensor,
    ad3: SimulatedSensor,

    // 数字输入（低频采样，10秒周期）
    di0: AtomicBool,
    di1: AtomicBool,
    last_digital_update: Mutex<Instant>,
    digital_toggle_interval: Duration,
}

impl SimulatedDataSource {
    pub fn new() -> Self {
        let ad1 = SimulatedSensor::new(SimulatorType::Random {
            min: 0.0,
            max: 4095.0,
        });
        let ad2 = SimulatedSensor::new(SimulatorType::Random {
            min: 0.0,
            max: 4095.0,
        });
        let ad3 = SimulatedSensor::new(SimulatorType::Random {
            min: 0.0,
            max: 4095.0,
        });

        Self {
            ad1,
            ad2,
            ad3,
            di0: AtomicBool::new(false),
            di1: AtomicBool::new(false),
            last_digital_update: Mutex::new(Instant::now()),
            digital_toggle_interval: Duration::from_secs(10),
        }
    }

    pub fn with_config(
        ad1_config: SimulatorType,
        ad2_config: SimulatorType,
        ad3_config: SimulatorType,
    ) -> Self {
        let ad1 = SimulatedSensor::new(ad1_config);
        let ad2 = SimulatedSensor::new(ad2_config);
        let ad3 = SimulatedSensor::new(ad3_config);

        Self {
            ad1,
            ad2,
            ad3,
            di0: AtomicBool::new(false),
            di1: AtomicBool::new(false),
            last_digital_update: Mutex::new(Instant::now()),
            digital_toggle_interval: Duration::from_secs(10),
        }
    }

    pub fn read_all(&self) -> SensorResult<(f64, f64, f64, bool, bool)> {
        let ad1 = self.ad1.read()?;
        let ad2 = self.ad2.read()?;
        let ad3 = self.ad3.read()?;

        let now = Instant::now();
        let should_toggle = {
            let mut last_update = self.last_digital_update.lock().unwrap();
            let elapsed = now.duration_since(*last_update);

            if elapsed >= self.digital_toggle_interval {
                *last_update = now;
                true
            } else {
                false
            }
        };

        if should_toggle {
            let current_di0 = self.di0.load(Ordering::SeqCst);
            let current_di1 = self.di1.load(Ordering::SeqCst);

            self.di0.store(!current_di0, Ordering::SeqCst);
            self.di1.store(!current_di1, Ordering::SeqCst);
        }

        let di0 = self.di0.load(Ordering::SeqCst);
        let di1 = self.di1.load(Ordering::SeqCst);

        Ok((ad1, ad2, ad3, di0, di1))
    }

    pub fn ad1(&self) -> &SimulatedSensor {
        &self.ad1
    }

    pub fn ad2(&self) -> &SimulatedSensor {
        &self.ad2
    }

    pub fn ad3(&self) -> &SimulatedSensor {
        &self.ad3
    }
}

impl SensorSource for SimulatedDataSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64, bool, bool)> {
        SimulatedDataSource::read_all(self)
    }

    fn is_connected(&self) -> bool {
        true
    }
}

impl Default for SimulatedDataSource {
    fn default() -> Self {
        Self::new()
    }
}
