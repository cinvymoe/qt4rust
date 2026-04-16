// src/pipeline/orchestration/mod.rs

mod manager;
mod orchestrator;

pub use manager::PipelineManager;
pub use orchestrator::{PipelineError, PipelineLifecycle, PipelineOrchestrator};
