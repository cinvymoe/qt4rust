use crate::error::SensorResult;

pub trait SensorProvider {
    fn read(&self) -> SensorResult<f64>;

    fn is_connected(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "Unknown Sensor"
    }
}

pub trait SensorSource {
    fn read_all(&self) -> SensorResult<(f64, f64, f64)>;
    fn is_connected(&self) -> bool;
}
