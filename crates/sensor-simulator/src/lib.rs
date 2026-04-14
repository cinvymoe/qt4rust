pub mod config;
pub mod data_source;
pub mod simulated;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::data_source::*;
    pub use crate::simulated::*;
    pub use crane_data_layer::prelude::*;
}

pub use crate::config::SimulatorConfig;
pub use crate::data_source::SimulatedDataSource;
pub use crate::simulated::{SimulatedSensor, SimulatorType};

#[deprecated(
    since = "0.2.0",
    note = "use SimulatedSensor with SimulatorType::Sine instead"
)]
pub type SineSimulator = SimulatedSensor;

#[deprecated(since = "0.2.0", note = "use crane_data_layer::SensorProvider instead")]
pub use crane_data_layer::traits::SensorProvider;

#[deprecated(since = "0.2.0", note = "use crane_data_layer::SensorResult instead")]
pub use crane_data_layer::error::SensorResult;

#[deprecated(since = "0.2.0", note = "use crane_data_layer::SensorError instead")]
pub use crane_data_layer::error::SensorError;
