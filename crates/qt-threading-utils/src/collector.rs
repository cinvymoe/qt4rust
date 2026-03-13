// Data collector utilities

use std::time::Duration;

pub struct DataCollector {
    interval: Duration,
}

impl DataCollector {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}
