// Sensor Error Types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SensorError {
    #[error("Failed to read sensor data: {0}")]
    ReadError(String),

    #[error("Failed to initialize sensor: {0}")]
    InitError(String),

    #[error("Sensor connection timeout")]
    Timeout,

    #[error("Invalid sensor configuration: {0}")]
    ConfigError(String),

    #[error("Sensor not connected")]
    NotConnected,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type SensorResult<T> = Result<T, SensorError>;
