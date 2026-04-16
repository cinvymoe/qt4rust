// src/pipeline/pipelines/mod.rs

mod collection_pipeline;
mod display_pipeline;
mod process_pipeline;
mod sensor_storage_pipeline;
mod storage_pipeline;

pub use collection_pipeline::{CollectionPipeline, CollectionPipelineConfig};
pub use display_pipeline::{DisplayPipeline, DisplayPipelineConfig};
pub use process_pipeline::{ProcessPipeline, ProcessPipelineConfig};
pub use sensor_storage_pipeline::SensorStoragePipeline;
pub use storage_pipeline::{StoragePipeline, StoragePipelineConfig};
