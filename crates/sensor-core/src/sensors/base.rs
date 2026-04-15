use crate::{SensorCalibrationParams, SensorProvider, SensorResult};

pub struct CalibratedSensor {
    calibration: SensorCalibrationParams,
    raw_provider: Box<dyn SensorProvider>,
}

impl CalibratedSensor {
    pub fn new(
        calibration: SensorCalibrationParams,
        raw_provider: Box<dyn SensorProvider>,
    ) -> Self {
        Self {
            calibration,
            raw_provider,
        }
    }

    pub fn read_calibrated(&self) -> SensorResult<f64> {
        let raw = self.raw_provider.read()?;
        Ok(self.calibration.convert_ad_to_value(raw))
    }
}
