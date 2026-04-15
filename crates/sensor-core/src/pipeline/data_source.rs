use std::fmt;

/// Identifies the source of sensor data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataSourceId {
    /// Data from Modbus TCP/RTU connection
    Modbus,
    /// Data from sensor simulator
    Simulator,
    /// Data from mock/test source
    Mock,
}

impl fmt::Display for DataSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSourceId::Modbus => write!(f, "Modbus"),
            DataSourceId::Simulator => write!(f, "Simulator"),
            DataSourceId::Mock => write!(f, "Mock"),
        }
    }
}

/// Raw sensor data with source identification.
#[derive(Debug, Clone)]
pub struct SourceSensorData {
    /// The source that produced this data
    pub source: DataSourceId,
    /// Raw weight sensor AD value
    pub weight_ad: u16,
    /// Raw angle sensor AD value
    pub angle_ad: u16,
    /// Raw radius sensor AD value
    pub radius_ad: u16,
    /// Timestamp in milliseconds since epoch
    pub timestamp_ms: u64,
}

impl SourceSensorData {
    /// Creates a new SourceSensorData instance.
    pub fn new(
        source: DataSourceId,
        weight_ad: u16,
        angle_ad: u16,
        radius_ad: u16,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            source,
            weight_ad,
            angle_ad,
            radius_ad,
            timestamp_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source_id_display() {
        assert_eq!(format!("{}", DataSourceId::Modbus), "Modbus");
        assert_eq!(format!("{}", DataSourceId::Simulator), "Simulator");
        assert_eq!(format!("{}", DataSourceId::Mock), "Mock");
    }

    #[test]
    fn test_data_source_id_equality() {
        assert_eq!(DataSourceId::Modbus, DataSourceId::Modbus);
        assert_ne!(DataSourceId::Modbus, DataSourceId::Simulator);
    }

    #[test]
    fn test_source_sensor_data_new() {
        let data = SourceSensorData::new(DataSourceId::Simulator, 1000, 2000, 3000, 1234567890);

        assert_eq!(data.source, DataSourceId::Simulator);
        assert_eq!(data.weight_ad, 1000);
        assert_eq!(data.angle_ad, 2000);
        assert_eq!(data.radius_ad, 3000);
        assert_eq!(data.timestamp_ms, 1234567890);
    }

    #[test]
    fn test_source_sensor_data_clone() {
        let data = SourceSensorData::new(DataSourceId::Mock, 100, 200, 300, 1000);
        let cloned = data.clone();

        assert_eq!(data.source, cloned.source);
        assert_eq!(data.weight_ad, cloned.weight_ad);
        assert_eq!(data.angle_ad, cloned.angle_ad);
        assert_eq!(data.radius_ad, cloned.radius_ad);
        assert_eq!(data.timestamp_ms, cloned.timestamp_ms);
    }
}
