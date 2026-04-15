use super::CalibratedSensor;
use crate::{SensorCalibrationParams, SensorProvider, SensorResult};

pub struct AngleSensor {
    inner: CalibratedSensor,
}

impl AngleSensor {
    pub fn new(
        calibration: SensorCalibrationParams,
        raw_provider: Box<dyn SensorProvider>,
    ) -> Self {
        Self {
            inner: CalibratedSensor::new(calibration, raw_provider),
        }
    }

    pub fn read_degrees(&self) -> SensorResult<f64> {
        self.inner.read_calibrated()
    }
}
