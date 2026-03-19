// 管道模块

pub mod shared_buffer;
pub mod collection_pipeline;
pub mod pipeline_manager;

// 重新导出常用类型
pub use shared_buffer::{ProcessedDataBuffer, SharedBuffer, BufferStats};
pub use collection_pipeline::{CollectionPipeline, CollectionPipelineConfig};
pub use pipeline_manager::PipelineManager;
