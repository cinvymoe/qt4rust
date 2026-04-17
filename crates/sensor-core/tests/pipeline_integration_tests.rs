//! Integration tests for the sensor pipeline system.
//!
//! These tests verify end-to-end data flow from sensor sources through
//! aggregation to storage.

use sensor_core::{
    DataSourceId, MockStorageRepository, PipelineConfig, SensorPipelineManager, SensorResult,
    SensorSource, StoragePipelineConfig,
};
use sensor_traits::SensorReading;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Mock sensor source for testing.
///
/// Provides sequential data values and can be configured with specific test data.
struct TestMockSensorSource {
    data: Vec<(f64, f64, f64, bool, bool)>,
    current_index: AtomicUsize,
}

impl TestMockSensorSource {
    fn new(data: Vec<(f64, f64, f64, bool, bool)>) -> Self {
        Self {
            data,
            current_index: AtomicUsize::new(0),
        }
    }
}

impl SensorSource for TestMockSensorSource {
    fn read_all(&self) -> SensorResult<SensorReading> {
        let index = self.current_index.fetch_add(1, Ordering::SeqCst);
        if index < self.data.len() {
            Ok(SensorReading::from_tuple(self.data[index].0, self.data[index].1, self.data[index].2, self.data[index].3, self.data[index].4))
        } else {
            let last = self.data.last().unwrap_or(&(0.0, 0.0, 0.0, false, false));
            Ok(SensorReading::from_tuple(last.0, last.1, last.2, last.3, last.4))
        }
    }

    fn is_connected(&self) -> bool {
        !self.data.is_empty()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_full_pipeline_with_single_source() {
    let mut manager = SensorPipelineManager::new();

    let source = Arc::new(TestMockSensorSource::new(vec![
        (100.0, 50.0, 45.0, false, false),
        (101.0, 51.0, 46.0, false, false),
        (102.0, 52.0, 47.0, false, false),
    ]));

    manager.register_source(
        DataSourceId::Mock,
        source,
        PipelineConfig {
            read_interval: Duration::from_millis(50),
            max_retries: 3,
            debug_logging: false,
        },
    );

    let repository = Arc::new(MockStorageRepository::new());
    manager.set_storage_config(StoragePipelineConfig {
        storage_interval: Duration::from_millis(100),
        batch_size: 10,
        enable_compression: false,
    });
    manager.set_storage_repository(repository.clone());

    manager.start_all().expect("Failed to start pipelines");
    tokio::time::sleep(Duration::from_millis(500)).await;
    manager.stop_all();

    let stored = repository.get_stored_data().await;
    assert!(!stored.is_empty(), "Should have stored some data");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_full_pipeline_with_multiple_sources() {
    let mut manager = SensorPipelineManager::new();

    let modbus_source = Arc::new(TestMockSensorSource::new(vec![
        (100.0, 50.0, 45.0, false, false),
        (101.0, 50.5, 45.5, false, false),
    ]));

    let simulator_source = Arc::new(TestMockSensorSource::new(vec![
        (100.5, 50.2, 45.2, false, false),
        (101.5, 50.7, 45.7, false, false),
    ]));

    manager.register_source(
        DataSourceId::Modbus,
        modbus_source,
        PipelineConfig::default(),
    );
    manager.register_source(
        DataSourceId::Simulator,
        simulator_source,
        PipelineConfig::default(),
    );

    let repository = Arc::new(MockStorageRepository::new());
    manager.set_storage_config(StoragePipelineConfig {
        storage_interval: Duration::from_millis(100),
        batch_size: 2,
        enable_compression: false,
    });
    manager.set_storage_repository(repository.clone());

    manager.start_all().expect("Failed to start pipelines");
    tokio::time::sleep(Duration::from_millis(500)).await;
    manager.stop_all();

    let stored = repository.get_stored_data().await;
    assert!(!stored.is_empty(), "Should have stored some data");

    for record in &stored {
        assert!(
            record.valid_source_count > 0,
            "Each record should have at least one valid source"
        );
    }
}
