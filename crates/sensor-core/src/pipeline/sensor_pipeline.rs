use crate::error::{PipelineError, SensorError};
use crate::pipeline::{DataSourceId, PipelineConfig, SourceSensorData};
use crate::traits::SensorSource;
use qt_threading_utils::runtime::spawn;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::{block_in_place, JoinHandle};

/// A pipeline that reads data from a sensor source at regular intervals.
pub struct SensorPipeline<S: SensorSource + Send + Sync + 'static> {
    /// Unique identifier for the data source
    id: DataSourceId,
    /// The sensor source to read from
    source: Arc<S>,
    /// Pipeline configuration
    config: PipelineConfig,
    /// Channel transmitter for sending sensor data
    tx: mpsc::Sender<SourceSensorData>,
    /// Flag indicating if the pipeline is running
    running: Arc<AtomicBool>,
    /// Handle to the spawned tokio task
    handle: Option<JoinHandle<()>>,
}

impl<S: SensorSource + Send + Sync + 'static> SensorPipeline<S> {
    /// Creates a new SensorPipeline.
    pub fn new(
        id: DataSourceId,
        source: Arc<S>,
        config: PipelineConfig,
        tx: mpsc::Sender<SourceSensorData>,
    ) -> Self {
        Self {
            id,
            source,
            config,
            tx,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Starts the pipeline, spawning a tokio task to read sensor data.
    /// Returns an error if the pipeline is already running.
    pub fn start(&mut self) -> Result<(), SensorError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(SensorError::Pipeline(PipelineError::AlreadyRunning));
        }

        self.running.store(true, Ordering::SeqCst);

        let source = Arc::clone(&self.source);
        let tx = self.tx.clone();
        let running = Arc::clone(&self.running);
        let interval_duration = self.config.read_interval;
        let max_retries = self.config.max_retries;
        let source_id = self.id;

        let handle = spawn(async move {
            let mut interval = tokio::time::interval(interval_duration);

            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }

                interval.tick().await;

                // Read with retry logic
                let mut retry_count = 0;
                let data = loop {
                    match block_in_place(|| source.read_all()) {
                        Ok((weight, angle, radius)) => {
                            break Ok((weight, angle, radius));
                        }
                        Err(e) => {
                            retry_count += 1;
                            if retry_count >= max_retries {
                                break Err(e);
                            }
                            // Brief delay before retry
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    }
                };

                match data {
                    Ok((weight, angle, radius)) => {
                        let sensor_data = SourceSensorData::new(
                            source_id,
                            weight as u16,
                            angle as u16,
                            radius as u16,
                            false,
                            false,
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                        );

                        if tx.send(sensor_data).await.is_err() {
                            // Channel closed, stop the pipeline
                            running.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                    Err(_e) => {
                        // Log error but continue running
                        // In production, this could trigger an alert
                        continue;
                    }
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stops the pipeline, aborting the running task.
    pub fn stop(&mut self) -> Result<(), SensorError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(SensorError::Pipeline(PipelineError::NotRunning));
        }

        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }

        Ok(())
    }

    /// Returns true if the pipeline is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::MockSensorSource;
    use std::time::Duration;
    use tokio::sync::mpsc;

    fn create_test_pipeline(
        data: Vec<(f64, f64, f64)>,
    ) -> (SensorPipeline<MockSensorSource>, mpsc::Receiver<SourceSensorData>) {
        let (tx, rx) = mpsc::channel(100);
        let source = Arc::new(MockSensorSource::new(data));
        let config = PipelineConfig {
            read_interval: Duration::from_millis(10),
            max_retries: 3,
            debug_logging: false,
        };
        let pipeline = SensorPipeline::new(DataSourceId::Mock, source, config, tx);
        (pipeline, rx)
    }

    #[test]
    fn test_pipeline_creation() {
        let (pipeline, _rx) = create_test_pipeline(vec![(1.0, 2.0, 3.0)]);

        assert_eq!(pipeline.id, DataSourceId::Mock);
        assert!(!pipeline.is_running());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_pipeline_start_sets_running() {
        let (mut pipeline, _rx) = create_test_pipeline(vec![(1.0, 2.0, 3.0)]);

        let result = pipeline.start();
        assert!(result.is_ok());
        assert!(pipeline.is_running());

        let _ = pipeline.stop();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_pipeline_start_twice_fails() {
        let (mut pipeline, _rx) = create_test_pipeline(vec![(1.0, 2.0, 3.0)]);

        let result1 = pipeline.start();
        assert!(result1.is_ok());

        let result2 = pipeline.start();
        assert!(result2.is_err());
        match result2 {
            Err(SensorError::Pipeline(PipelineError::AlreadyRunning)) => {}
            _ => panic!("Expected AlreadyRunning error"),
        }

        let _ = pipeline.stop();
    }

    #[test]
    fn test_pipeline_stop_when_not_running_fails() {
        let (mut pipeline, _rx) = create_test_pipeline(vec![(1.0, 2.0, 3.0)]);

        let result = pipeline.stop();
        assert!(result.is_err());
        match result {
            Err(SensorError::Pipeline(PipelineError::NotRunning)) => {}
            _ => panic!("Expected NotRunning error"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_pipeline_sends_data_through_channel() {
        let (mut pipeline, mut rx) = create_test_pipeline(vec![(100.0, 200.0, 300.0)]);

        pipeline.start().unwrap();

        let data = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("Should receive data within timeout")
            .expect("Should have data");

        assert_eq!(data.source, DataSourceId::Mock);
        assert_eq!(data.weight_ad, 100);
        assert_eq!(data.angle_ad, 200);
        assert_eq!(data.radius_ad, 300);

        let _ = pipeline.stop();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_pipeline_stop_aborts_task() {
        let (mut pipeline, _rx) = create_test_pipeline(vec![(1.0, 2.0, 3.0)]);

        pipeline.start().unwrap();
        assert!(pipeline.is_running());

        pipeline.stop().unwrap();
        assert!(!pipeline.is_running());
    }

    #[test]
    fn test_pipeline_config_clone() {
        let config = PipelineConfig::default();
        let cloned = config.clone();

        assert_eq!(config.read_interval, cloned.read_interval);
        assert_eq!(config.max_retries, cloned.max_retries);
    }
}
