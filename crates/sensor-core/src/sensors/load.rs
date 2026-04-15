use super::CalibratedSensor;
use crate::{SensorCalibrationParams, SensorProvider, SensorResult};

pub struct LoadSensor {
    inner: CalibratedSensor,
}

impl LoadSensor {
    pub fn new(
        calibration: SensorCalibrationParams,
        raw_provider: Box<dyn SensorProvider>,
    ) -> Self {
        Self {
            inner: CalibratedSensor::new(calibration, raw_provider),
        }
    }

    pub fn read_tons(&self) -> SensorResult<f64> {
        self.inner.read_calibrated()
    }
}
