use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Pipeline {0} already running")]
    AlreadyRunning(String),

    #[error("Pipeline {0} not found")]
    NotFound(String),

    #[error("Pipeline {0} start failed: {1}")]
    StartFailed(String, String),
}

pub trait PipelineLifecycle: Send {
    fn pipeline_id(&self) -> &str;

    fn start(&mut self) -> Result<(), PipelineError>;

    fn stop(&mut self);

    fn is_running(&self) -> bool;
}

pub struct PipelineOrchestrator {
    pipelines: HashMap<String, Box<dyn PipelineLifecycle>>,
    startup_order: Vec<String>,
    shutdown_order: Vec<String>,
}

impl PipelineOrchestrator {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            startup_order: Vec::new(),
            shutdown_order: Vec::new(),
        }
    }

    pub fn register(&mut self, pipeline: Box<dyn PipelineLifecycle>) {
        let id = pipeline.pipeline_id().to_string();

        if !self.startup_order.contains(&id) {
            self.startup_order.push(id.clone());
        }
        if !self.shutdown_order.contains(&id) {
            self.shutdown_order.insert(0, id.clone());
        }

        self.pipelines.insert(id, pipeline);
    }

    pub fn set_startup_order(&mut self, order: Vec<&str>) {
        self.startup_order = order.iter().map(|s| s.to_string()).collect();
    }

    pub fn set_shutdown_order(&mut self, order: Vec<&str>) {
        self.shutdown_order = order.iter().map(|s| s.to_string()).collect();
    }

    pub fn start_pipeline(&mut self, id: &str) -> Result<(), PipelineError> {
        let pipeline = self
            .pipelines
            .get_mut(id)
            .ok_or_else(|| PipelineError::NotFound(id.to_string()))?;

        if pipeline.is_running() {
            return Err(PipelineError::AlreadyRunning(id.to_string()));
        }

        pipeline.start()?;
        info!("Pipeline '{}' started", id);
        Ok(())
    }

    pub fn stop_pipeline(&mut self, id: &str) {
        if let Some(pipeline) = self.pipelines.get_mut(id) {
            pipeline.stop();
            info!("Pipeline '{}' stopped", id);
        } else {
            warn!("Pipeline '{}' not found for stop", id);
        }
    }

    pub fn start_all(&mut self) -> Result<(), PipelineError> {
        info!("Starting all pipelines in order: {:?}", self.startup_order);

        for id in &self.startup_order.clone() {
            if let Some(pipeline) = self.pipelines.get_mut(id) {
                if !pipeline.is_running() {
                    pipeline.start()?;
                    info!("Pipeline '{}' started", id);
                }
            }
        }

        info!("All pipelines started");
        Ok(())
    }

    pub fn stop_all(&mut self) {
        info!("Stopping all pipelines in order: {:?}", self.shutdown_order);

        for id in &self.shutdown_order.clone() {
            if let Some(pipeline) = self.pipelines.get_mut(id) {
                if pipeline.is_running() {
                    pipeline.stop();
                    info!("Pipeline '{}' stopped", id);
                }
            }
        }

        info!("All pipelines stopped");
    }

    pub fn is_running(&self, id: &str) -> bool {
        self.pipelines
            .get(id)
            .map(|p| p.is_running())
            .unwrap_or(false)
    }

    pub fn is_any_running(&self) -> bool {
        self.pipelines.values().any(|p| p.is_running())
    }

    pub fn running_count(&self) -> usize {
        self.pipelines.values().filter(|p| p.is_running()).count()
    }

    pub fn count(&self) -> usize {
        self.pipelines.len()
    }

    pub fn pipeline_ids(&self) -> Vec<&str> {
        self.pipelines.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PipelineOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPipeline {
        running: bool,
        name: String,
    }

    impl PipelineLifecycle for MockPipeline {
        fn pipeline_id(&self) -> &str {
            &self.name
        }

        fn start(&mut self) -> Result<(), PipelineError> {
            self.running = true;
            Ok(())
        }

        fn stop(&mut self) {
            self.running = false;
        }

        fn is_running(&self) -> bool {
            self.running
        }
    }

    #[test]
    fn test_orchestrator_register() {
        let mut orchestrator = PipelineOrchestrator::new();

        let pipeline = MockPipeline {
            running: false,
            name: "test".to_string(),
        };

        orchestrator.register(Box::new(pipeline));
        assert_eq!(orchestrator.count(), 1);
    }

    #[test]
    fn test_orchestrator_start_all() {
        let mut orchestrator = PipelineOrchestrator::new();

        orchestrator.register(Box::new(MockPipeline {
            running: false,
            name: "p1".to_string(),
        }));
        orchestrator.register(Box::new(MockPipeline {
            running: false,
            name: "p2".to_string(),
        }));

        let result = orchestrator.start_all();
        assert!(result.is_ok());
        assert!(orchestrator.is_any_running());
    }

    #[test]
    fn test_orchestrator_stop_all() {
        let mut orchestrator = PipelineOrchestrator::new();

        orchestrator.register(Box::new(MockPipeline {
            running: false,
            name: "p1".to_string(),
        }));

        orchestrator.start_all().unwrap();
        orchestrator.stop_all();

        assert!(!orchestrator.is_any_running());
    }
}
