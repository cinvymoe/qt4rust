use crate::simulated::{SimulatedSensor, SimulatorType};
use crane_data_layer::prelude::*;

pub struct SimulatedDataSource {
    ad1: SimulatedSensor,
    ad2: SimulatedSensor,
    ad3: SimulatedSensor,
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

        Self { ad1, ad2, ad3 }
    }

    pub fn with_config(
        ad1_config: SimulatorType,
        ad2_config: SimulatorType,
        ad3_config: SimulatorType,
    ) -> Self {
        let ad1 = SimulatedSensor::new(ad1_config);
        let ad2 = SimulatedSensor::new(ad2_config);
        let ad3 = SimulatedSensor::new(ad3_config);

        Self { ad1, ad2, ad3 }
    }

    pub fn read_all(&self) -> SensorResult<(f64, f64, f64)> {
        let ad1 = self.ad1.read()?;
        let ad2 = self.ad2.read()?;
        let ad3 = self.ad3.read()?;
        Ok((ad1, ad2, ad3))
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
    fn read_all(&self) -> SensorResult<(f64, f64, f64)> {
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
