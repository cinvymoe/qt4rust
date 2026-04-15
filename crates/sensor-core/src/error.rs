use std::fmt;

#[derive(Debug, Clone)]
pub enum SensorError {
    ReadError(String),
    InitError(String),
    Timeout,
    ConfigError(String),
    NotConnected,
    IoError(String),
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadError(msg) => write!(f, "读取传感器数据失败: {}", msg),
            Self::InitError(msg) => write!(f, "初始化传感器失败: {}", msg),
            Self::Timeout => write!(f, "传感器连接超时"),
            Self::ConfigError(msg) => write!(f, "传感器配置错误: {}", msg),
            Self::NotConnected => write!(f, "传感器未连接"),
            Self::IoError(msg) => write!(f, "I/O 错误: {}", msg),
        }
    }
}

impl std::error::Error for SensorError {}

pub type SensorResult<T> = Result<T, SensorError>;
