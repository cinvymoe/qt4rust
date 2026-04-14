// Modbus TCP Client Library

pub mod config;
pub mod error;
pub mod client;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::error::*;
    pub use crate::client::*;
}
