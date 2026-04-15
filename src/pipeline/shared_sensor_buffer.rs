// 共享传感器数据缓冲区

use crate::models::SensorData;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Thread-safe buffer for latest SensorData
#[derive(Debug)]
pub struct SensorDataBuffer {
    latest: Option<SensorData>,
    history: VecDeque<SensorData>,
    max_history: usize,
}

impl SensorDataBuffer {
    /// Create new buffer with specified max history size
    pub fn new(max_history: usize) -> Self {
        Self {
            latest: None,
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Push new sensor data into buffer
    pub fn push(&mut self, data: SensorData) {
        self.latest = Some(data.clone());

        // Limit history size (FIFO)
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(data);
    }

    /// Get the latest sensor data
    pub fn get_latest(&self) -> Option<SensorData> {
        self.latest.clone()
    }

    /// Get the latest raw values as (ad1_load, ad2_radius, ad3_angle) tuple
    pub fn get_latest_raw(&self) -> Option<(f64, f64, f64)> {
        self.latest
            .as_ref()
            .map(|data| (data.ad1_load, data.ad2_radius, data.ad3_angle))
    }

    /// Get recent history (up to count items, newest first)
    pub fn get_history(&self, count: usize) -> Vec<SensorData> {
        self.history.iter().rev().take(count).cloned().collect()
    }
}

/// Thread-safe shared buffer type
pub type SharedSensorBuffer = Arc<RwLock<SensorDataBuffer>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = SensorDataBuffer::new(10);
        assert!(buffer.get_latest().is_none());
        assert!(buffer.get_latest_raw().is_none());
        assert!(buffer.get_history(10).is_empty());
    }

    #[test]
    fn test_push_and_get_latest() {
        let mut buffer = SensorDataBuffer::new(10);
        let data = SensorData::new(100.0, 50.0, 45.0, false, false);

        buffer.push(data.clone());

        let latest = buffer.get_latest();
        assert!(latest.is_some());
        let latest = latest.unwrap();
        assert_eq!(latest.ad1_load, 100.0);
        assert_eq!(latest.ad2_radius, 50.0);
        assert_eq!(latest.ad3_angle, 45.0);
    }

    #[test]
    fn test_get_latest_raw() {
        let mut buffer = SensorDataBuffer::new(10);
        let data = SensorData::new(200.0, 75.0, 60.0, false, false);

        buffer.push(data);

        let raw = buffer.get_latest_raw();
        assert!(raw.is_some());
        let (ad1, ad2, ad3) = raw.unwrap();
        assert_eq!(ad1, 200.0);
        assert_eq!(ad2, 75.0);
        assert_eq!(ad3, 60.0);
    }

    #[test]
    fn test_capacity_limit() {
        let mut buffer = SensorDataBuffer::new(5);

        // Add 10 items
        for i in 0..10 {
            buffer.push(SensorData::new(
                i as f64,
                (i * 2) as f64,
                (i * 3) as f64,
                false,
                false,
            ));
        }

        // Should only keep last 5
        let history = buffer.get_history(10);
        assert_eq!(history.len(), 5);
        assert_eq!(history[0].ad1_load, 9.0); // newest
        assert_eq!(history[4].ad1_load, 5.0); // oldest retained
    }

    #[test]
    fn test_get_history_order() {
        let mut buffer = SensorDataBuffer::new(10);

        for i in 0..5 {
            buffer.push(SensorData::new(i as f64, 0.0, 0.0, false, false));
        }

        // Get history returns newest first
        let history = buffer.get_history(3);
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].ad1_load, 4.0); // newest
        assert_eq!(history[1].ad1_load, 3.0);
        assert_eq!(history[2].ad1_load, 2.0);
    }

    #[test]
    fn test_latest_updates_correctly() {
        let mut buffer = SensorDataBuffer::new(10);

        buffer.push(SensorData::new(1.0, 1.0, 1.0, false, false));
        assert_eq!(buffer.get_latest_raw().unwrap().0, 1.0);

        buffer.push(SensorData::new(2.0, 2.0, 2.0, false, false));
        assert_eq!(buffer.get_latest_raw().unwrap().0, 2.0);

        buffer.push(SensorData::new(3.0, 3.0, 3.0, false, false));
        assert_eq!(buffer.get_latest_raw().unwrap().0, 3.0);
    }

    #[test]
    fn test_shared_sensor_buffer_type() {
        let buffer = SensorDataBuffer::new(10);
        let shared: SharedSensorBuffer = Arc::new(RwLock::new(buffer));

        // Test that we can write and read through the lock
        {
            let mut locked = shared.write().unwrap();
            locked.push(SensorData::new(42.0, 24.0, 12.0, false, false));
        }

        {
            let locked = shared.read().unwrap();
            let raw = locked.get_latest_raw();
            assert!(raw.is_some());
            assert_eq!(raw.unwrap().0, 42.0);
        }
    }
}
