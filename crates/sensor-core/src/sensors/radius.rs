use super::CalibratedSensor;
use crate::{SensorCalibrationParams, SensorProvider, SensorResult};

pub struct RadiusSensor {
    inner: CalibratedSensor,
}

impl RadiusSensor {
    pub fn new(
        calibration: SensorCalibrationParams,
        raw_provider: Box<dyn SensorProvider>,
    ) -> Self {
        Self {
            inner: CalibratedSensor::new(calibration, raw_provider),
        }
    }

    pub fn read_meters(&self) -> SensorResult<f64> {
        self.inner.read_calibrated()
    }
}
