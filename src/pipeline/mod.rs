// 管道模块

use crate::models::ProcessedData;

pub mod collection_pipeline;
pub mod display_pipeline;
pub mod event_channel;
pub mod filter_buffer;
pub mod pipeline_manager;
pub mod process_pipeline;
pub mod retry_policy;
pub mod sensor_data_event_channel;
pub mod sensor_storage_pipeline;
pub mod shared_buffer;
pub mod shared_sensor_buffer;
pub mod storage_pipeline;
pub mod storage_queue;

/// Unified storage events for event-driven architecture
#[derive(Debug, Clone)]
pub enum StorageEvent {
    /// New data available for storage
    NewData(Vec<ProcessedData>),

    /// Alarm triggered
    Alarm(ProcessedData),

    /// Alarm cleared (danger → safe transition)
    AlarmCleared,

    /// Request graceful shutdown
    Shutdown,
}

/// Errors for event-driven storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Channel send error: {0}")]
    ChannelSend(String),
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Queue full, would block")]
    QueueFull,
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

pub use collection_pipeline::{CollectionPipeline, CollectionPipelineConfig};
pub use display_pipeline::{DisplayPipeline, DisplayPipelineConfig};
pub use event_channel::{create_storage_channels, StorageEventReceiver, StorageEventSender};
pub use filter_buffer::{FilterBuffer, FilterBufferConfig, FilterType};
pub use pipeline_manager::PipelineManager;
pub use process_pipeline::{ProcessPipeline, ProcessPipelineConfig};
pub use retry_policy::{with_retry, RetryConfig};
pub use sensor_data_event_channel::{
    create_sensor_data_channels, SensorDataEventReceiver, SensorDataEventSender,
};
pub use sensor_storage_pipeline::SensorStoragePipeline;
pub use shared_buffer::{BufferStats, ProcessedDataBuffer, SharedBuffer};
pub use shared_sensor_buffer::{SensorDataBuffer, SharedSensorBuffer};
pub use storage_pipeline::{StoragePipeline, StoragePipelineConfig};
pub use storage_queue::StorageQueue;
