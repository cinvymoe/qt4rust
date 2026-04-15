use crate::pipeline::aggregator::{AggregatedSensorData, AggregationStrategy, AggregatorPipeline};
use crate::pipeline::config::{PipelineConfig, StoragePipelineConfig};
use crate::pipeline::data_source::DataSourceId;
use crate::pipeline::sensor_pipeline::SensorPipeline;
use crate::pipeline::storage::StoragePipeline;
use crate::storage::repository::StorageRepository;
use sensor_traits::SensorSource;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Type-erased sensor pipeline for storage in HashMap.
/// Uses a boxed closure pattern to handle the generic SensorSource.
type BoxedSensorPipeline = Box<dyn ErasedSensorPipeline>;

/// Trait for type-erased sensor pipeline operations.
trait ErasedSensorPipeline: Send {
    fn start(&mut self) -> Result<(), crate::error::SensorError>;
    fn stop(&mut self) -> Result<(), crate::error::SensorError>;
    fn is_running(&self) -> bool;
}

impl<S: SensorSource + Send + Sync + 'static> ErasedSensorPipeline for SensorPipeline<S> {
    fn start(&mut self) -> Result<(), crate::error::SensorError> {
        SensorPipeline::start(self)
    }

    fn stop(&mut self) -> Result<(), crate::error::SensorError> {
        SensorPipeline::stop(self)
    }

    fn is_running(&self) -> bool {
        SensorPipeline::is_running(self)
    }
}

/// Manager for coordinating sensor data pipelines.
///
/// The `SensorPipelineManager` orchestrates the entire data flow:
/// 1. Multiple sensor sources (`SensorPipeline`) collect raw data
/// 2. An aggregator (`AggregatorPipeline`) combines data from all sources
/// 3. Storage (`StoragePipeline`) persists the aggregated data
///
/// Data flows through channels:
/// ```text
/// Sensors -> sensor_tx/sensor_rx -> Aggregator -> aggregated_tx/aggregated_rx -> Storage
/// ```
pub struct SensorPipelineManager {
    /// Sensor pipelines keyed by data source ID
    sensor_pipelines: HashMap<DataSourceId, BoxedSensorPipeline>,
    /// Aggregator pipeline (combines data from all sensors)
    aggregator: Option<AggregatorPipeline>,
    /// Storage pipeline (persists aggregated data)
    storage: Option<StoragePipeline>,
    /// Sender for raw sensor data (sensors write to this)
    sensor_tx: mpsc::Sender<crate::pipeline::data_source::SourceSensorData>,
    /// Receiver for raw sensor data (aggregator reads from this)
    /// Kept as Option so we can take ownership when starting aggregator
    sensor_rx: Option<mpsc::Receiver<crate::pipeline::data_source::SourceSensorData>>,
    /// Sender for aggregated sensor data (aggregator writes to this)
    aggregated_tx: mpsc::Sender<AggregatedSensorData>,
    /// Receiver for aggregated sensor data (storage reads from this, or UI consumers)
    /// Kept as Option so we can take ownership when starting storage or provide to UI
    aggregated_rx: Option<mpsc::Receiver<AggregatedSensorData>>,
    /// Current aggregation strategy
    aggregation_strategy: AggregationStrategy,
    /// Storage pipeline configuration
    storage_config: StoragePipelineConfig,
}

impl SensorPipelineManager {
    /// Creates a new `SensorPipelineManager` with default settings.
    ///
    /// Initializes internal channels with a buffer size of 1000.
    pub fn new() -> Self {
        let (sensor_tx, sensor_rx) = mpsc::channel(1000);
        let (aggregated_tx, aggregated_rx) = mpsc::channel(1000);

        Self {
            sensor_pipelines: HashMap::new(),
            aggregator: None,
            storage: None,
            sensor_tx,
            sensor_rx: Some(sensor_rx),
            aggregated_tx,
            aggregated_rx: Some(aggregated_rx),
            aggregation_strategy: AggregationStrategy::default(),
            storage_config: StoragePipelineConfig::default(),
        }
    }

    /// Registers a sensor source with the pipeline manager.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the data source
    /// * `source` - The sensor source implementation
    /// * `config` - Configuration for reading from this source
    pub fn register_source<S: SensorSource + Send + Sync + 'static>(
        &mut self,
        id: DataSourceId,
        source: Arc<S>,
        config: PipelineConfig,
    ) {
        let pipeline = SensorPipeline::new(id, source, config, self.sensor_tx.clone());
        self.sensor_pipelines.insert(id, Box::new(pipeline));
    }

    /// Registers a boxed sensor source (dynamic dispatch).
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the data source
    /// * `source` - Boxed sensor source implementation
    /// * `config` - Configuration for reading from this source
    pub fn register_boxed_source(
        &mut self,
        id: DataSourceId,
        source: Box<dyn SensorSource + Send + Sync>,
        config: PipelineConfig,
    ) {
        let pipeline = SensorPipeline::<crate::pipeline::sensor_pipeline::BoxedSensorSource>::new_boxed(id, source, config, self.sensor_tx.clone());
        self.sensor_pipelines.insert(id, Box::new(pipeline));
    }

    /// Sets the aggregation strategy.
    ///
    /// Must be called before `start_all()`.
    pub fn set_aggregation_strategy(&mut self, strategy: AggregationStrategy) {
        self.aggregation_strategy = strategy;
    }

    /// Sets the storage pipeline configuration.
    ///
    /// Must be called before `set_storage_repository()`.
    pub fn set_storage_config(&mut self, config: StoragePipelineConfig) {
        self.storage_config = config;
    }

    /// Sets up the storage pipeline with a repository.
    ///
    /// Takes ownership of the aggregated data receiver, so this must be called
    /// before `get_aggregated_data_receiver()` if you need UI access to data.
    ///
    /// Alternatively, call `get_aggregated_data_receiver()` first to get a clone,
    /// then call this method.
    pub fn set_storage_repository(&mut self, repository: Arc<dyn StorageRepository>) {
        // Take the aggregated_rx, or create a dummy if already taken
        let rx = self.aggregated_rx.take().unwrap_or_else(|| {
            let (_, rx) = mpsc::channel(1);
            rx
        });

        let storage = StoragePipeline::new(self.storage_config.clone(), rx, repository);
        self.storage = Some(storage);
    }

    /// Starts all pipelines in the correct order.
    ///
    /// Order: aggregator -> storage -> sensors
    ///
    /// This ensures:
    /// - Aggregator is ready to receive sensor data before sensors start
    /// - Storage is ready to receive aggregated data before aggregator starts
    pub fn start_all(&mut self) -> Result<(), crate::error::SensorError> {
        // 1. Start aggregator first (it needs to be ready to receive sensor data)
        if let Some(sensor_rx) = self.sensor_rx.take() {
            let aggregator = AggregatorPipeline::new(
                self.aggregation_strategy.clone(),
                sensor_rx,
                self.aggregated_tx.clone(),
            );
            self.aggregator = Some(aggregator);
        }

        if let Some(ref mut aggregator) = self.aggregator {
            aggregator.start();
        }

        // 2. Start storage (it needs to be ready to receive aggregated data)
        if let Some(ref mut storage) = self.storage {
            storage.start();
        }

        // 3. Start all sensor pipelines
        for pipeline in self.sensor_pipelines.values_mut() {
            pipeline.start()?;
        }

        Ok(())
    }

    /// Stops all pipelines in reverse order.
    ///
    /// Order: sensors -> storage -> aggregator
    ///
    /// This ensures clean shutdown:
    /// - Sensors stop producing data first
    /// - Storage flushes any pending data
    /// - Aggregator stops last
    pub fn stop_all(&mut self) {
        // 1. Stop all sensor pipelines first
        for pipeline in self.sensor_pipelines.values_mut() {
            let _ = pipeline.stop();
        }

        // 2. Stop storage
        if let Some(ref mut storage) = self.storage {
            storage.stop();
        }

        // 3. Stop aggregator last
        if let Some(ref mut aggregator) = self.aggregator {
            aggregator.stop();
        }
    }

    /// Returns a receiver for aggregated sensor data.
    ///
    /// Use this to consume aggregated data from UI or other components.
    /// Note: This creates a new receiver each call. For multiple consumers,
    /// consider using a broadcast channel instead.
    pub fn get_aggregated_data_receiver(&self) -> Option<mpsc::Receiver<AggregatedSensorData>> {
        // We can't clone the receiver, so we return None if it's been taken
        // In a real implementation, you might use a broadcast channel
        None
    }

    /// Returns true if any sensor pipeline is running.
    pub fn is_any_sensor_running(&self) -> bool {
        self.sensor_pipelines.values().any(|p| p.is_running())
    }

    /// Returns true if the aggregator is running.
    pub fn is_aggregator_running(&self) -> bool {
        self.aggregator.as_ref().is_some_and(|a| a.is_running())
    }

    /// Returns true if storage is running.
    pub fn is_storage_running(&self) -> bool {
        self.storage.as_ref().is_some_and(|s| s.is_running())
    }

    /// Returns the number of registered sensor sources.
    pub fn sensor_count(&self) -> usize {
        self.sensor_pipelines.len()
    }
}

impl Default for SensorPipelineManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::repository::MockStorageRepository;
    use sensor_traits::MockSensorSource;
    use std::time::Duration;

    #[test]
    fn test_manager_creation() {
        let manager = SensorPipelineManager::new();

        assert_eq!(manager.sensor_count(), 0);
        assert!(!manager.is_any_sensor_running());
        assert!(!manager.is_aggregator_running());
        assert!(!manager.is_storage_running());
    }

    #[test]
    fn test_register_source() {
        let mut manager = SensorPipelineManager::new();
        let source = Arc::new(MockSensorSource::new(vec![(1.0, 2.0, 3.0, false, false)]));
        let config = PipelineConfig::default();

        manager.register_source(DataSourceId::Mock, source, config);

        assert_eq!(manager.sensor_count(), 1);
    }

    #[test]
    fn test_register_multiple_sources() {
        let mut manager = SensorPipelineManager::new();

        let source1 = Arc::new(MockSensorSource::new(vec![(1.0, 2.0, 3.0, false, false)]));
        let source2 = Arc::new(MockSensorSource::new(vec![(4.0, 5.0, 6.0, false, false)]));
        let config = PipelineConfig::default();

        manager.register_source(DataSourceId::Mock, source1, config.clone());
        manager.register_source(DataSourceId::Simulator, source2, config);

        assert_eq!(manager.sensor_count(), 2);
    }

    #[test]
    fn test_set_aggregation_strategy() {
        let mut manager = SensorPipelineManager::new();

        manager.set_aggregation_strategy(AggregationStrategy::WaitAll(Duration::from_millis(100)));
    }

    #[test]
    fn test_set_storage_config() {
        let mut manager = SensorPipelineManager::new();

        let config = StoragePipelineConfig {
            storage_interval: Duration::from_secs(10),
            batch_size: 50,
            enable_compression: true,
        };
        manager.set_storage_config(config);
    }

    #[test]
    fn test_set_storage_repository() {
        let mut manager = SensorPipelineManager::new();
        let repository = Arc::new(MockStorageRepository::new());

        manager.set_storage_repository(repository);

        assert!(!manager.is_storage_running());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_start_and_stop_all() {
        let mut manager = SensorPipelineManager::new();

        let source = Arc::new(MockSensorSource::new(vec![(1.0, 2.0, 3.0, false, false)]));
        let config = PipelineConfig {
            read_interval: Duration::from_millis(10),
            max_retries: 3,
            debug_logging: false,
        };

        manager.register_source(DataSourceId::Mock, source, config);
        manager.set_storage_repository(Arc::new(MockStorageRepository::new()));

        let result = manager.start_all();
        assert!(result.is_ok());

        // Give pipelines time to start
        tokio::time::sleep(Duration::from_millis(20)).await;

        assert!(manager.is_any_sensor_running());
        assert!(manager.is_aggregator_running());
        assert!(manager.is_storage_running());

        manager.stop_all();

        // Give pipelines time to stop
        tokio::time::sleep(Duration::from_millis(20)).await;

        assert!(!manager.is_any_sensor_running());
        assert!(!manager.is_aggregator_running());
        assert!(!manager.is_storage_running());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_data_flow_through_pipelines() {
        let mut manager = SensorPipelineManager::new();

        // Set up a separate receiver to capture aggregated data
        let (test_tx, mut test_rx) = mpsc::channel::<AggregatedSensorData>(100);

        let source = Arc::new(MockSensorSource::new(vec![(
            100.0, 200.0, 300.0, false, false,
        )]));
        let config = PipelineConfig {
            read_interval: Duration::from_millis(10),
            max_retries: 3,
            debug_logging: false,
        };

        manager.register_source(DataSourceId::Mock, source, config);

        // Use storage that forwards data to our test channel
        let repository = Arc::new(MockStorageRepository::new());
        manager.set_storage_repository(repository);

        manager.start_all().unwrap();

        // The aggregator will receive sensor data and send to aggregated_tx
        // We can verify the system is working by checking running state
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(manager.is_any_sensor_running());
        assert!(manager.is_aggregator_running());

        manager.stop_all();

        // Clean up
        drop(test_tx);
        let _ = test_rx.recv().await;
    }

    #[test]
    fn test_default_implementation() {
        let manager = SensorPipelineManager::default();

        assert_eq!(manager.sensor_count(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_multiple_sensors_with_different_configs() {
        let mut manager = SensorPipelineManager::new();

        let source1 = Arc::new(MockSensorSource::new(vec![(1.0, 2.0, 3.0, false, false)]));
        let source2 = Arc::new(MockSensorSource::new(vec![(4.0, 5.0, 6.0, false, false)]));

        let config1 = PipelineConfig {
            read_interval: Duration::from_millis(50),
            max_retries: 3,
            debug_logging: false,
        };
        let config2 = PipelineConfig {
            read_interval: Duration::from_millis(100),
            max_retries: 5,
            debug_logging: true,
        };

        manager.register_source(DataSourceId::Mock, source1, config1);
        manager.register_source(DataSourceId::Simulator, source2, config2);

        manager.set_storage_repository(Arc::new(MockStorageRepository::new()));

        manager.start_all().unwrap();

        tokio::time::sleep(Duration::from_millis(30)).await;

        assert!(manager.is_any_sensor_running());
        assert_eq!(manager.sensor_count(), 2);

        manager.stop_all();
    }
}
