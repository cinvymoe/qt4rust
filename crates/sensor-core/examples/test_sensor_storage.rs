use sensor_core::{
    init_builtin_sources, AggregationStrategy, DataSourceId, MockStorageRepository, PipelineConfig,
    SensorPipelineManager, SensorSource, SensorSourceFactory, StoragePipelineConfig,
};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

fn setup_data_source(config_path: &Path) -> Box<dyn SensorSource> {
    init_builtin_sources();

    if config_path.exists() {
        println!("[INFO] Using Modbus TCP config: {}", config_path.display());
    } else {
        println!("[INFO] No Modbus config found, using simulator");
    }

    SensorSourceFactory::create_from_config(config_path)
}

fn setup_pipeline_manager(
    source: Box<dyn SensorSource>,
    repository: Arc<MockStorageRepository>,
) -> SensorPipelineManager {
    let mut manager = SensorPipelineManager::new();

    // Register data source
    manager.register_boxed_source(
        DataSourceId::Modbus,
        source,
        PipelineConfig {
            read_interval: Duration::from_millis(100), // 10 Hz
            max_retries: 3,
            debug_logging: false,
        },
    );

    // Configure aggregation (Immediate for responsive testing)
    manager.set_aggregation_strategy(AggregationStrategy::Immediate);

    // Configure storage
    manager.set_storage_config(StoragePipelineConfig {
        storage_interval: Duration::from_secs(1), // Flush every second
        batch_size: 100,
        enable_compression: false,
    });

    // Set storage repository
    manager.set_storage_repository(repository);

    println!("[INFO] Pipeline configured:");
    println!("  - Read interval: 100ms");
    println!("  - Storage interval: 1s");
    println!("  - Aggregation: Immediate");

    manager
}

fn run_collection(manager: &mut SensorPipelineManager, duration: Duration) {
    println!("\n[INFO] Starting pipeline...");
    manager.start_all().expect("Failed to start pipelines");

    println!("[INFO] Collecting data for {:?}...\n", duration);

    let start = std::time::Instant::now();
    let mut last_report = start;

    while start.elapsed() < duration {
        std::thread::sleep(Duration::from_millis(100));

        // Report progress every second
        if last_report.elapsed() >= Duration::from_secs(1) {
            let elapsed = start.elapsed();
            let remaining = duration.saturating_sub(elapsed);
            println!(
                "[{:.1}s] Collecting... ({:.1}s remaining)",
                elapsed.as_secs_f64(),
                remaining.as_secs_f64()
            );
            last_report = std::time::Instant::now();
        }
    }

    println!("\n[INFO] Stopping pipeline...");
    manager.stop_all();
    println!("[INFO] Pipeline stopped");
}

fn verify_storage(repository: &MockStorageRepository) -> bool {
    println!("\n=== Verification Results ===\n");

    // Use block_on for the async call since we're in sync context
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let stored = rt.block_on(repository.get_stored_data());
    let count = stored.len();

    println!("Total stored records: {}", count);

    if count == 0 {
        println!("[ERROR] No data stored!");
        return false;
    }

    // Calculate expected count (10s / 100ms read interval ≈ 100 records)
    // Account for timing variations and batch processing
    let expected_min = 80; // Allow 20% variance
    let expected_max = 120;

    if count < expected_min {
        println!(
            "[WARN] Stored count ({}) below expected minimum ({})",
            count, expected_min
        );
    } else if count > expected_max {
        println!(
            "[WARN] Stored count ({}) above expected maximum ({})",
            count, expected_max
        );
    } else {
        println!(
            "[OK] Stored count within expected range [{}, {}]",
            expected_min, expected_max
        );
    }

    // Validate each record
    let mut validation_errors = 0;
    let mut prev_timestamp: Option<std::time::Instant> = None;
    let mut timestamp_errors = 0;

    for (i, record) in stored.iter().enumerate() {
        // Check data validity
        for (source_id, sensor_data) in &record.sources {
            if let Err(e) = sensor_data.validate() {
                validation_errors += 1;
                if validation_errors <= 5 {
                    // Only print first 5
                    println!(
                        "[ERROR] Record {} validation failed ({}): {}",
                        i, source_id, e
                    );
                }
            }
        }

        // Check timestamp monotonicity
        if let Some(prev) = prev_timestamp {
            if record.timestamp < prev {
                timestamp_errors += 1;
            }
        }
        prev_timestamp = Some(record.timestamp);
    }

    println!("\nValidation summary:");
    println!("  - Records validated: {}", count);
    println!("  - Validation errors: {}", validation_errors);
    println!("  - Timestamp errors: {}", timestamp_errors);

    // Sample output
    if !stored.is_empty() {
        println!("\nSample records:");
        for (i, record) in stored.iter().take(3).enumerate() {
            if let Some((source_id, data)) = record.sources.iter().next() {
                println!(
                    "[{}] {} -> weight={:.1}, radius={:.1}, angle={:.1}, di0={}, di1={}",
                    i,
                    source_id,
                    data.ad1_load(),
                    data.ad2_radius(),
                    data.ad3_angle(),
                    data.digital_input_0(),
                    data.digital_input_1()
                );
            }
        }
    }

    let success = validation_errors == 0 && timestamp_errors == 0 && count >= expected_min;

    if success {
        println!("\n[PASS] All verification checks passed!");
    } else {
        println!("\n[FAIL] Some verification checks failed!");
    }

    success
}

fn main() {
    println!("=== Sensor Storage Test Example ===\n");

    // Setup data source (sync - before any runtime)
    let config_path = Path::new("config/modbus_sensors.toml");
    let source = setup_data_source(config_path);

    // Create shared storage repository
    let repository = Arc::new(MockStorageRepository::new());
    println!("[INFO] MockStorageRepository created");

    // Setup pipeline
    let mut manager = setup_pipeline_manager(source, Arc::clone(&repository));

    // Run collection for 10 seconds
    run_collection(&mut manager, Duration::from_secs(10));

    // Verify results
    let success = verify_storage(&repository);

    println!("\n=== Test Complete ===");
    std::process::exit(if success { 0 } else { 1 });
}
