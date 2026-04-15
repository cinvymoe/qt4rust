use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SensorError {
    #[error("读取传感器数据失败: {0}")]
    ReadError(String),
    #[error("初始化传感器失败: {0}")]
    InitError(String),
    #[error("传感器连接超时")]
    Timeout,
    #[error("传感器配置错误: {0}")]
    ConfigError(String),
    #[error("传感器未连接")]
    NotConnected,
    #[error("I/O 错误: {0}")]
    IoError(String),
    #[error("{0}")]
    Pipeline(#[from] PipelineError),
    #[error("{0}")]
    Storage(#[from] StorageError),
}

pub type SensorResult<T> = Result<T, SensorError>;

#[derive(Debug, Clone, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Sequence conflict")]
    SequenceConflict,
    #[error("Health check failed")]
    HealthCheckFailed,
}

#[derive(Debug, Clone, Error)]
pub enum PipelineError {
    #[error("Pipeline already running")]
    AlreadyRunning,
    #[error("Pipeline not running")]
    NotRunning,
    #[error("Data source error: {data_source} - {error}")]
    DataSource { data_source: String, error: String },
    #[error("Channel send error")]
    ChannelSend,
    #[error("Channel receive error")]
    ChannelRecv,
}
