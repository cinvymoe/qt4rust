// Modbus TCP Error Types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModbusError {
    #[error("Failed to read data: {0}")]
    ReadError(String),

    #[error("Failed to initialize: {0}")]
    InitError(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Not connected")]
    NotConnected,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

pub type ModbusResult<T> = Result<T, ModbusError>;
