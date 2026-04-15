//! Integration tests for the sensor pipeline system.
//!
//! These tests verify end-to-end data flow from sensor sources through
//! aggregation to storage.

use sensor_core::{
    DataSourceId, PipelineConfig,
    SensorPipelineManager, SensorSource, SensorResult, StoragePipelineConfig,
    MockStorageRepository,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Mock sensor source for testing.
/// 
/// Provides sequential data values and can be configured with specific test data.
struct TestMockSensorSource {
    data: Vec<(f64, f64, f64)>,
    current_index: AtomicUsize,
}

impl TestMockSensorSource {
    fn new(data: Vec<(f64, f64, f64)>) -> Self {
        Self {
            data,
            current_index: AtomicUsize::new(0),
        }
    }
}

impl SensorSource for TestMockSensorSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64)> {
        let index = self.current_index.fetch_add(1, Ordering::SeqCst);
        if index < self.data.len() {
            Ok(self.data[index])
        } else {
            Ok(*self.data.last().unwrap_or(&(0.0, 0.0, 0.0)))
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
        (100.0, 50.0, 45.0),
        (101.0, 51.0, 46.0),
        (102.0, 52.0, 47.0),
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
        (100.0, 50.0, 45.0),
        (101.0, 50.5, 45.5),
    ]));

    let simulator_source = Arc::new(TestMockSensorSource::new(vec![
        (100.5, 50.2, 45.2),
        (101.5, 50.7, 45.7),
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
