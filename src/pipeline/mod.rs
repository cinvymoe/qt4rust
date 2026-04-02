// 管道模块

pub mod shared_buffer;
pub mod collection_pipeline;
pub mod display_pipeline;
pub mod pipeline_manager;
pub mod storage_queue;
pub mod storage_pipeline;

// 重新导出常用类型
pub use shared_buffer::{ProcessedDataBuffer, SharedBuffer, BufferStats};
pub use collection_pipeline::{CollectionPipeline, CollectionPipelineConfig};
pub use display_pipeline::{DisplayPipeline, DisplayPipelineConfig};
pub use pipeline_manager::PipelineManager;
pub use storage_queue::StorageQueue;
pub use storage_pipeline::{StoragePipeline, StoragePipelineConfig};
