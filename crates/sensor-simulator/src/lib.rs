pub mod config;
pub mod data_source;
pub mod simulated;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::data_source::*;
    pub use crate::simulated::*;
    pub use sensor_core::{SensorError, SensorProvider, SensorResult, SensorSource};
}

pub use crate::config::SimulatorConfig;
pub use crate::data_source::SimulatedDataSource;
pub use crate::simulated::{SimulatedSensor, SimulatorType};

#[deprecated(
    since = "0.2.0",
    note = "use SimulatedSensor with SimulatorType::Sine instead"
)]
pub type SineSimulator = SimulatedSensor;

#[deprecated(since = "0.3.0", note = "use sensor_core::SensorProvider instead")]
pub use sensor_core::SensorProvider;

#[deprecated(since = "0.3.0", note = "use sensor_core::SensorResult instead")]
pub use sensor_core::SensorResult;

#[deprecated(since = "0.3.0", note = "use sensor_core::SensorError instead")]
pub use sensor_core::SensorError;
