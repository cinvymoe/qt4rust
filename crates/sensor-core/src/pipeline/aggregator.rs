use crate::data::sensor_data::SensorData;
use crate::pipeline::data_source::{DataSourceId, SourceSensorData};
use qt_threading_utils::runtime::spawn;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AggregatedSensorData {
    pub sources: HashMap<DataSourceId, SensorData>,
    pub timestamp: Instant,
    pub valid_source_count: usize,
}

impl AggregatedSensorData {
    pub fn new(sources: HashMap<DataSourceId, SensorData>) -> Self {
        let valid_source_count = sources.len();
        Self {
            sources,
            timestamp: Instant::now(),
            valid_source_count,
        }
    }

    pub fn add_source(&mut self, source: DataSourceId, data: SensorData) {
        self.sources.insert(source, data);
        self.valid_source_count = self.sources.len();
    }

    pub fn get_source(&self, source: DataSourceId) -> Option<&SensorData> {
        self.sources.get(&source)
    }
}

#[derive(Debug, Clone, Default)]
pub enum AggregationStrategy {
    #[default]
    Immediate,
    WaitAll(std::time::Duration),
    PrimaryBackup {
        primary: DataSourceId,
        backup: Vec<DataSourceId>,
    },
}

/// Pipeline that aggregates sensor data from multiple sources.
pub struct AggregatorPipeline {
    /// Aggregation strategy to use
    strategy: AggregationStrategy,
    /// Receiver for incoming source sensor data
    rx: mpsc::Receiver<SourceSensorData>,
    /// Sender for aggregated sensor data
    tx: mpsc::Sender<AggregatedSensorData>,
    /// Flag indicating if the pipeline is running
    running: Arc<AtomicBool>,
    /// Handle to the spawned task
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl AggregatorPipeline {
    /// Creates a new AggregatorPipeline.
    pub fn new(
        strategy: AggregationStrategy,
        rx: mpsc::Receiver<SourceSensorData>,
        tx: mpsc::Sender<AggregatedSensorData>,
    ) -> Self {
        Self {
            strategy,
            rx,
            tx,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Starts the aggregator pipeline, spawning a background task.
    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);
        let tx = self.tx.clone();
        let strategy = self.strategy.clone();

        let mut rx = std::mem::replace(&mut self.rx, {
            let (_, dummy_rx) = mpsc::channel(1);
            dummy_rx
        });

        let handle = spawn(async move {
            let mut cache: HashMap<DataSourceId, (SensorData, Instant)> = HashMap::new();

            while running.load(Ordering::SeqCst) {
                match rx.recv().await {
                    Some(data) => {
                        let sensor_data = SensorData::new(
                            data.weight_ad as f64,
                            data.radius_ad as f64,
                            data.angle_ad as f64,
                            data.digital_input_0,
                            data.digital_input_1,
                        );

                        cache.insert(data.source, (sensor_data.clone(), Instant::now()));

                        match &strategy {
                            AggregationStrategy::Immediate => {
                                let sources: HashMap<DataSourceId, SensorData> =
                                    cache.iter().map(|(k, (v, _))| (*k, v.clone())).collect();

                                let aggregated = AggregatedSensorData::new(sources);

                                if tx.send(aggregated).await.is_err() {
                                    break;
                                }
                            }
                            AggregationStrategy::WaitAll(_duration) => {
                                let sources: HashMap<DataSourceId, SensorData> =
                                    cache.iter().map(|(k, (v, _))| (*k, v.clone())).collect();

                                let aggregated = AggregatedSensorData::new(sources);

                                if tx.send(aggregated).await.is_err() {
                                    break;
                                }
                            }
                            AggregationStrategy::PrimaryBackup { primary, backup: _ } => {
                                if let Some((data, _)) = cache.get(primary) {
                                    let mut sources = HashMap::new();
                                    sources.insert(*primary, data.clone());
                                    let aggregated = AggregatedSensorData::new(sources);

                                    if tx.send(aggregated).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Stops the aggregator pipeline.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregated_sensor_data_creation() {
        let mut sources = HashMap::new();
        sources.insert(
            DataSourceId::Modbus,
            SensorData::new(100.0, 50.0, 45.0, false, false),
        );
        let aggregated = AggregatedSensorData::new(sources);
        assert_eq!(aggregated.valid_source_count, 1);
    }

    #[test]
    fn test_aggregated_sensor_data_add_source() {
        let mut aggregated = AggregatedSensorData::new(HashMap::new());
        aggregated.add_source(
            DataSourceId::Modbus,
            SensorData::new(100.0, 50.0, 45.0, false, false),
        );
        aggregated.add_source(
            DataSourceId::Simulator,
            SensorData::new(101.0, 51.0, 46.0, false, false),
        );
        assert_eq!(aggregated.valid_source_count, 2);
    }

    #[tokio::test]
    async fn test_aggregator_pipeline_immediate_strategy() {
        let (input_tx, input_rx) = mpsc::channel::<SourceSensorData>(10);
        let (output_tx, mut output_rx) = mpsc::channel::<AggregatedSensorData>(10);

        let mut pipeline =
            AggregatorPipeline::new(AggregationStrategy::Immediate, input_rx, output_tx);

        assert!(!pipeline.is_running());
        pipeline.start();
        assert!(pipeline.is_running());

        let data = SourceSensorData::new(DataSourceId::Modbus, 100, 200, 300, false, false, 1000);
        input_tx.send(data).await.unwrap();

        let aggregated = output_rx
            .recv()
            .await
            .expect("Should receive aggregated data");
        assert_eq!(aggregated.valid_source_count, 1);
        assert!(aggregated.sources.contains_key(&DataSourceId::Modbus));

        pipeline.stop();
        assert!(!pipeline.is_running());
    }

    #[tokio::test]
    async fn test_aggregator_pipeline_multiple_sources() {
        let (input_tx, input_rx) = mpsc::channel::<SourceSensorData>(10);
        let (output_tx, mut output_rx) = mpsc::channel::<AggregatedSensorData>(10);

        let mut pipeline =
            AggregatorPipeline::new(AggregationStrategy::Immediate, input_rx, output_tx);

        pipeline.start();

        let data1 = SourceSensorData::new(DataSourceId::Modbus, 100, 200, 300, false, false, 1000);
        input_tx.send(data1).await.unwrap();
        let _ = output_rx.recv().await;

        let data2 =
            SourceSensorData::new(DataSourceId::Simulator, 150, 250, 350, false, false, 2000);
        input_tx.send(data2).await.unwrap();

        let aggregated = output_rx
            .recv()
            .await
            .expect("Should receive aggregated data");
        assert_eq!(aggregated.valid_source_count, 2);
        assert!(aggregated.sources.contains_key(&DataSourceId::Modbus));
        assert!(aggregated.sources.contains_key(&DataSourceId::Simulator));

        pipeline.stop();
    }

    #[tokio::test]
    async fn test_aggregator_pipeline_primary_backup_strategy() {
        let (input_tx, input_rx) = mpsc::channel::<SourceSensorData>(10);
        let (output_tx, mut output_rx) = mpsc::channel::<AggregatedSensorData>(10);

        let strategy = AggregationStrategy::PrimaryBackup {
            primary: DataSourceId::Modbus,
            backup: vec![DataSourceId::Simulator],
        };

        let mut pipeline = AggregatorPipeline::new(strategy, input_rx, output_tx);
        pipeline.start();

        let backup_data =
            SourceSensorData::new(DataSourceId::Simulator, 150, 250, 350, false, false, 1000);
        input_tx.send(backup_data).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let primary_data =
            SourceSensorData::new(DataSourceId::Modbus, 100, 200, 300, false, false, 2000);
        input_tx.send(primary_data).await.unwrap();

        let aggregated = output_rx
            .recv()
            .await
            .expect("Should receive aggregated data");
        assert_eq!(aggregated.valid_source_count, 1);
        assert!(aggregated.sources.contains_key(&DataSourceId::Modbus));

        pipeline.stop();
    }
}
