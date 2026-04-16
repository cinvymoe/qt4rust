pub mod core;
pub mod infrastructure;
pub mod orchestration;
pub mod pipelines;
pub mod services;

pub use core::{StorageError, StorageEvent};
pub use infrastructure::{
    BufferStats, EventBus, EventBusChannels, PipelineEvent, ProcessedDataBuffer,
    SensorDataBuffer, SensorDataEventReceiver, SensorDataEventSender, SharedBuffer,
    SharedSensorBuffer, StorageQueue, StorageEventReceiver, StorageEventSender,
    RetryConfig, create_sensor_data_channels, create_storage_channels, with_retry,
};
pub use orchestration::{PipelineError, PipelineLifecycle, PipelineOrchestrator, PipelineManager};
pub use pipelines::{
    CollectionPipeline, CollectionPipelineConfig, DisplayPipeline, DisplayPipelineConfig,
    ProcessPipeline, ProcessPipelineConfig, SensorStoragePipeline, StoragePipeline,
    StoragePipelineConfig,
};
pub use services::{
    AlarmAction, AlarmDebounceConfig, AlarmDebouncer, CalibrationService, ConfigProvider,
    FilterBuffer, FilterBufferConfig, FilterType, StorageService, StorageServiceConfig,
};
