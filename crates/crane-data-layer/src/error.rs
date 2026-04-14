use std::fmt;

#[derive(Debug, Clone)]
pub enum DataError {
    SourceUnavailable(String),
    NotFound(String),
    ValidationError(String),
    CacheError(String),
    SerializationError(String),
    IoError(String),
    Timeout(String),
    PermissionDenied(String),
    Unknown(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SourceUnavailable(msg) => write!(f, "数据源不可用: {}", msg),
            Self::NotFound(msg) => write!(f, "数据未找到: {}", msg),
            Self::ValidationError(msg) => write!(f, "数据验证失败: {}", msg),
            Self::CacheError(msg) => write!(f, "缓存错误: {}", msg),
            Self::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            Self::IoError(msg) => write!(f, "I/O 错误: {}", msg),
            Self::Timeout(msg) => write!(f, "超时: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "权限不足: {}", msg),
            Self::Unknown(msg) => write!(f, "未知错误: {}", msg),
        }
    }
}

impl std::error::Error for DataError {}

pub type DataResult<T> = Result<T, DataError>;

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
