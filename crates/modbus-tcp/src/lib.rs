pub mod client;
pub mod config;
pub mod data_source;
pub mod error;

pub use data_source::{ModbusDataSource, SensorKind, SensorModbusConfig};

pub mod prelude {
    pub use crate::client::*;
    pub use crate::config::*;
    pub use crate::data_source::*;
    pub use crate::error::*;
}
