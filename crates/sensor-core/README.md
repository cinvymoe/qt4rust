# sensor-core

Core sensor abstraction layer with multi-source pipeline architecture.

## Features

- **Multi-source support**: Independent pipelines for each data source (Modbus, Simulator, Mock), each running on its own Tokio task
- **Data aggregation**: Merge data from multiple sources with configurable strategies (Immediate, WaitAll, PrimaryBackup)
- **Storage abstraction**: Trait-based storage backend (`StorageRepository`) with built-in mock for testing. Swap between SQLite, PostgreSQL, or in-memory without changing pipeline code
- **Async pipeline**: Built on Tokio for high-performance async processing with channel-based data flow
- **Retry logic**: Configurable retry attempts on communication failure per source
- **Batch persistence**: Storage pipeline batches writes by size or time interval, with a final flush on shutdown

## Architecture

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Modbus      │  │  Simulator  │  │  Mock       │
│  Source      │  │  Source     │  │  Source      │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Sensor     │  │  Sensor     │  │  Sensor      │
│  Pipeline   │  │  Pipeline   │  │  Pipeline    │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                  │                  │
       └──────────┬───────┴──────────────────┘
                  │  mpsc channel
                  ▼
          ┌───────────────┐
          │  Aggregator   │
          │  Pipeline     │
          └───────┬───────┘
                  │  mpsc channel
                  ▼
          ┌───────────────┐
          │  Storage      │
          │  Pipeline     │
          └───────┬───────┘
                  │
                  ▼
          ┌───────────────┐
          │  Storage      │
          │  Repository   │
          │  (trait)      │
          └───────────────┘
```

### Data flow

1. Each `SensorPipeline` reads from its `SensorSource` at a configurable interval, tags the reading with a `DataSourceId`, and sends it through an `mpsc` channel.
2. The `AggregatorPipeline` receives raw `SourceSensorData` from all sources, applies the configured `AggregationStrategy`, and emits `AggregatedSensorData`.
3. The `StoragePipeline` batches aggregated data and persists it through a `StorageRepository` implementation.

### Aggregation strategies

| Strategy | Behavior |
|----------|----------|
| `Immediate` | Emits aggregated data as soon as any source reports |
| `WaitAll(duration)` | Waits up to `duration` for all registered sources before emitting |
| `PrimaryBackup { primary, backup }` | Emits data from the primary source; falls back to backup sources if primary is unavailable |

### Key types

| Type | Role |
|------|------|
| `SensorPipelineManager` | Orchestrates all pipelines: register sources, configure aggregation, set storage, start/stop |
| `SensorPipeline<S>` | Reads from a `SensorSource` on a timed loop with retry logic |
| `AggregatorPipeline` | Merges data from multiple sources according to `AggregationStrategy` |
| `StoragePipeline` | Batches and persists `AggregatedSensorData` via `StorageRepository` |
| `DataSourceId` | Enum identifying the source: `Modbus`, `Simulator`, or `Mock` |
| `SourceSensorData` | Raw reading tagged with its source and timestamp |
| `AggregatedSensorData` | Merged view of data from one or more sources |
| `PipelineConfig` | Per-source settings: read interval, max retries, debug logging |
| `StoragePipelineConfig` | Storage settings: interval, batch size, compression toggle |

## Usage

### Basic setup

```rust
use sensor_core::{
    AggregationStrategy, DataSourceId, PipelineConfig, SensorPipelineManager,
    StoragePipelineConfig,
};
use std::sync::Arc;
use std::time::Duration;

let mut manager = SensorPipelineManager::new();

// Register a data source
manager.register_source(
    DataSourceId::Modbus,
    Arc::new(my_modbus_source),
    PipelineConfig {
        read_interval: Duration::from_millis(100),
        max_retries: 3,
        debug_logging: false,
    },
);

// Configure aggregation
manager.set_aggregation_strategy(AggregationStrategy::WaitAll(Duration::from_millis(50)));

// Configure storage
manager.set_storage_config(StoragePipelineConfig {
    storage_interval: Duration::from_secs(5),
    batch_size: 100,
    enable_compression: false,
});
manager.set_storage_repository(Arc::new(my_storage_impl));

// Start all pipelines
manager.start_all()?;

// ... run your application ...

// Clean shutdown (stops sensors first, then storage, then aggregator)
manager.stop_all();
```

### Implementing a custom data source

```rust
use sensor_core::{SensorSource, SensorResult};

struct MyModbusSource {
    // your connection state
}

impl SensorSource for MyModbusSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64)> {
        // Read weight, angle, radius from your hardware
        Ok((weight, angle, radius))
    }

    fn is_connected(&self) -> bool {
        // Check connection state
        true
    }
}
```

### Implementing a custom storage backend

```rust
use sensor_core::{AggregatedSensorData, StorageRepository, StorageError};
use async_trait::async_trait;

struct SqliteStorage {
    // your database pool
}

#[async_trait]
impl StorageRepository for SqliteStorage {
    async fn save_aggregated_data_batch(
        &self,
        data: Vec<AggregatedSensorData>,
    ) -> Result<(), StorageError> {
        // Persist the batch to SQLite
        Ok(())
    }

    async fn query_recent_aggregated_data(
        &self,
        limit: usize,
    ) -> Result<Vec<AggregatedSensorData>, StorageError> {
        // Query recent data
        Ok(vec![])
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        // Verify database connectivity
        Ok(())
    }

    async fn get_last_sequence(&self) -> Result<u64, StorageError> {
        // Return the last stored sequence number
        Ok(0)
    }
}
```

## Module layout

```
src/
├── lib.rs                  # Public API re-exports
├── error.rs                # SensorError, StorageError, PipelineError
├── traits.rs               # SensorSource, SensorProvider traits
├── algorithms/
│   └── ad_converter.rs     # AD value conversion
├── calibration/
│   └── sensor_calibration.rs  # Two-point calibration
├── data/
│   └── sensor_data.rs      # SensorData struct
├── sensors/
│   ├── base.rs             # CalibratedSensor wrapper
│   ├── angle.rs            # Angle sensor
│   ├── radius.rs           # Radius sensor
│   └── load.rs             # Load/weight sensor
├── pipeline/
│   ├── mod.rs              # Pipeline module re-exports
│   ├── manager.rs          # SensorPipelineManager
│   ├── sensor_pipeline.rs  # Per-source pipeline
│   ├── aggregator.rs        # AggregatorPipeline + AggregationStrategy
│   ├── storage.rs          # StoragePipeline
│   ├── config.rs           # PipelineConfig, StoragePipelineConfig
│   └── data_source.rs      # DataSourceId, SourceSensorData
└── storage/
    ├── mod.rs
    └── repository.rs       # StorageRepository trait + MockStorageRepository
```

## Running tests

```bash
# From the workspace root
cargo test -p sensor-core

# With output
cargo test -p sensor-core -- --nocapture
```