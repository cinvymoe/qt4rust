pub mod algorithms;
pub mod calibration;
pub mod data;
pub mod error;
pub mod sensors;
pub mod traits;

pub use algorithms::ad_converter::AdConverter;
pub use calibration::sensor_calibration::{
    AlarmThresholds, AngleThresholds, HookSwitchMode, HookSwitchThresholds, MomentThresholds,
    SensorCalibration, SensorCalibrationParams,
};
pub use data::sensor_data::SensorData;
pub use error::{SensorError, SensorResult};
pub use sensors::{AngleSensor, CalibratedSensor, LoadSensor, RadiusSensor};
pub use traits::{SensorProvider, SensorSource};
