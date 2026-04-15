use thiserror::Error;

pub use sensor_traits::{SensorError, SensorResult};

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

impl From<PipelineError> for SensorError {
    fn from(err: PipelineError) -> Self {
        SensorError::Pipeline(err.to_string())
    }
}

impl From<StorageError> for SensorError {
    fn from(err: StorageError) -> Self {
        SensorError::Storage(err.to_string())
    }
}
