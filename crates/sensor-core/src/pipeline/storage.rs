use crate::error::StorageError;
use crate::pipeline::aggregator::AggregatedSensorData;
use crate::pipeline::config::StoragePipelineConfig;
use crate::storage::repository::StorageRepository;
use qt_threading_utils::runtime::spawn;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::interval;

/// Pipeline that batches and persists aggregated sensor data.
pub struct StoragePipeline {
    /// Configuration for the storage pipeline
    config: StoragePipelineConfig,
    /// Receiver for incoming aggregated sensor data
    rx: mpsc::Receiver<AggregatedSensorData>,
    /// Repository for persisting data
    repository: Arc<dyn StorageRepository>,
    /// Pending batch of data to be persisted
    pending_batch: Vec<AggregatedSensorData>,
    /// Sequence number for ordering
    sequence: Arc<AtomicU64>,
    /// Flag indicating if the pipeline is running
    running: Arc<AtomicBool>,
    /// Handle to the spawned task
    handle: Option<JoinHandle<()>>,
}

impl StoragePipeline {
    /// Creates a new StoragePipeline.
    pub fn new(
        config: StoragePipelineConfig,
        rx: mpsc::Receiver<AggregatedSensorData>,
        repository: Arc<dyn StorageRepository>,
    ) -> Self {
        Self {
            config,
            rx,
            repository,
            pending_batch: Vec::new(),
            sequence: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Sets the initial sequence number.
    pub fn set_initial_sequence(&mut self, sequence: u64) {
        self.sequence.store(sequence, Ordering::SeqCst);
    }

    /// Starts the storage pipeline, spawning a background task.
    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);
        let repository = Arc::clone(&self.repository);
        let sequence = Arc::clone(&self.sequence);
        let batch_size = self.config.batch_size;
        let storage_interval = self.config.storage_interval;

        let mut rx = std::mem::replace(&mut self.rx, {
            let (_, dummy_rx) = mpsc::channel(1);
            dummy_rx
        });

        let handle = spawn(async move {
            let mut pending_batch: Vec<AggregatedSensorData> = Vec::new();
            let mut tick = interval(storage_interval);

            while running.load(Ordering::SeqCst) {
                tokio::select! {
                    _ = tick.tick() => {
                        if !pending_batch.is_empty() {
                            if let Err(e) = flush_batch(&repository, &mut pending_batch, &sequence).await {
                                eprintln!("Failed to flush batch on interval: {:?}", e);
                            }
                        }
                    }
                    Some(data) = rx.recv(), if running.load(Ordering::SeqCst) => {
                        pending_batch.push(data);

                        if pending_batch.len() >= batch_size {
                            if let Err(e) = flush_batch(&repository, &mut pending_batch, &sequence).await {
                                eprintln!("Failed to flush batch on size: {:?}", e);
                            }
                        }
                    }
                    else => {
                        break;
                    }
                }
            }

            if !pending_batch.is_empty() {
                if let Err(e) = flush_batch(&repository, &mut pending_batch, &sequence).await {
                    eprintln!("Failed final flush: {:?}", e);
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Stops the storage pipeline with a final flush.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    /// Returns true if the pipeline is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Returns the current pending batch size.
    pub fn pending_count(&self) -> usize {
        self.pending_batch.len()
    }
}

/// Flushes the pending batch to the repository.
async fn flush_batch(
    repository: &Arc<dyn StorageRepository>,
    batch: &mut Vec<AggregatedSensorData>,
    sequence: &Arc<AtomicU64>,
) -> Result<(), StorageError> {
    if batch.is_empty() {
        return Ok(());
    }

    let data_to_save = std::mem::take(batch);
    repository.save_aggregated_data_batch(data_to_save).await?;
    sequence.fetch_add(1, Ordering::SeqCst);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::sensor_data::SensorData;
    use crate::pipeline::data_source::DataSourceId;
    use crate::storage::repository::MockStorageRepository;
    use std::collections::HashMap;
    use std::time::Duration;

    fn create_test_aggregated_data(weight: f64) -> AggregatedSensorData {
        let mut sources = HashMap::new();
        sources.insert(
            DataSourceId::Simulator,
            SensorData::from_tuple(weight, 50.0, 45.0, false, false),
        );
        AggregatedSensorData::new(sources)
    }

    fn create_test_config() -> StoragePipelineConfig {
        StoragePipelineConfig {
            storage_interval: Duration::from_millis(50),
            batch_size: 3,
            enable_compression: false,
        }
    }

    #[test]
    fn test_storage_pipeline_creation() {
        let config = create_test_config();
        let (_, rx) = mpsc::channel::<AggregatedSensorData>(10);
        let repository = Arc::new(MockStorageRepository::new());

        let pipeline = StoragePipeline::new(config, rx, repository);

        assert!(!pipeline.is_running());
        assert_eq!(pipeline.pending_count(), 0);
    }

    #[test]
    fn test_set_initial_sequence() {
        let config = create_test_config();
        let (_, rx) = mpsc::channel::<AggregatedSensorData>(10);
        let repository = Arc::new(MockStorageRepository::new());

        let mut pipeline = StoragePipeline::new(config, rx, repository);
        pipeline.set_initial_sequence(42);
    }

    #[tokio::test]
    async fn test_start_and_stop() {
        let config = create_test_config();
        let (_, rx) = mpsc::channel::<AggregatedSensorData>(10);
        let repository = Arc::new(MockStorageRepository::new());

        let mut pipeline = StoragePipeline::new(config, rx, repository);

        assert!(!pipeline.is_running());
        pipeline.start();
        assert!(pipeline.is_running());

        tokio::time::sleep(Duration::from_millis(10)).await;

        pipeline.stop();
        assert!(!pipeline.is_running());
    }

    #[tokio::test]
    async fn test_batch_flush_on_size_reached() {
        let config = create_test_config();
        let (tx, rx) = mpsc::channel::<AggregatedSensorData>(10);
        let repository = Arc::new(MockStorageRepository::new());
        let stored_repo = Arc::clone(&repository);

        let mut pipeline = StoragePipeline::new(config, rx, repository);
        pipeline.start();

        for i in 0..3 {
            tx.send(create_test_aggregated_data(i as f64 * 100.0))
                .await
                .unwrap();
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        let stored = stored_repo.get_stored_data().await;
        assert_eq!(stored.len(), 3);

        pipeline.stop();
    }

    #[tokio::test]
    async fn test_batch_flush_on_interval() {
        let config = StoragePipelineConfig {
            storage_interval: Duration::from_millis(50),
            batch_size: 100,
            enable_compression: false,
        };
        let (tx, rx) = mpsc::channel::<AggregatedSensorData>(10);
        let repository = Arc::new(MockStorageRepository::new());
        let stored_repo = Arc::clone(&repository);

        let mut pipeline = StoragePipeline::new(config, rx, repository);
        pipeline.start();

        tx.send(create_test_aggregated_data(100.0)).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let stored = stored_repo.get_stored_data().await;
        assert_eq!(stored.len(), 1);

        pipeline.stop();
    }

    #[tokio::test]
    async fn test_final_flush_on_stop() {
        let config = StoragePipelineConfig {
            storage_interval: Duration::from_secs(10),
            batch_size: 100,
            enable_compression: false,
        };
        let (tx, rx) = mpsc::channel::<AggregatedSensorData>(10);
        let repository = Arc::new(MockStorageRepository::new());
        let stored_repo = Arc::clone(&repository);

        let mut pipeline = StoragePipeline::new(config, rx, repository);
        pipeline.start();

        tx.send(create_test_aggregated_data(100.0)).await.unwrap();

        tokio::time::sleep(Duration::from_millis(20)).await;

        pipeline.stop();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let stored = stored_repo.get_stored_data().await;
        assert_eq!(stored.len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_batches() {
        let config = create_test_config();
        let (tx, rx) = mpsc::channel::<AggregatedSensorData>(20);
        let repository = Arc::new(MockStorageRepository::new());
        let stored_repo = Arc::clone(&repository);

        let mut pipeline = StoragePipeline::new(config, rx, repository);
        pipeline.start();

        for i in 0..7 {
            tx.send(create_test_aggregated_data(i as f64 * 100.0))
                .await
                .unwrap();
        }

        tokio::time::sleep(Duration::from_millis(200)).await;

        let stored = stored_repo.get_stored_data().await;
        assert_eq!(stored.len(), 7);

        pipeline.stop();
    }
}
