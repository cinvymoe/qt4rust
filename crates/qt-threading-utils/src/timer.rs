// Timer utilities

use std::time::Duration;

pub struct PeriodicTimer {
    interval: Duration,
}

impl PeriodicTimer {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}
