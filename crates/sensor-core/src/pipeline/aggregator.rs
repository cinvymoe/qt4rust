use crate::data::sensor_data::SensorData;
use crate::pipeline::data_source::DataSourceId;
use std::collections::HashMap;
use std::time::Instant;

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

#[derive(Debug, Clone)]
pub enum AggregationStrategy {
    Immediate,
    WaitAll(std::time::Duration),
    PrimaryBackup {
        primary: DataSourceId,
        backup: Vec<DataSourceId>,
    },
}

impl Default for AggregationStrategy {
    fn default() -> Self {
        AggregationStrategy::Immediate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregated_sensor_data_creation() {
        let mut sources = HashMap::new();
        sources.insert(DataSourceId::Modbus, SensorData::new(100.0, 50.0, 45.0));
        let aggregated = AggregatedSensorData::new(sources);
        assert_eq!(aggregated.valid_source_count, 1);
    }

    #[test]
    fn test_aggregated_sensor_data_add_source() {
        let mut aggregated = AggregatedSensorData::new(HashMap::new());
        aggregated.add_source(DataSourceId::Modbus, SensorData::new(100.0, 50.0, 45.0));
        aggregated.add_source(DataSourceId::Simulator, SensorData::new(101.0, 51.0, 46.0));
        assert_eq!(aggregated.valid_source_count, 2);
    }
}
