use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SensorError {
    #[error("读取传感器数据失败: {0}")]
    ReadError(String),
    #[error("初始化传感器失败: {0}")]
    InitError(String),
    #[error("传感器连接超时")]
    Timeout,
    #[error("传感器配置错误: {0}")]
    ConfigError(String),
    #[error("传感器未连接")]
    NotConnected,
    #[error("I/O 错误: {0}")]
    IoError(String),
    #[error("Pipeline 错误: {0}")]
    Pipeline(String),
    #[error("Storage 错误: {0}")]
    Storage(String),
}

pub type SensorResult<T> = Result<T, SensorError>;

pub trait AnalogSource: Send + Sync {
    fn read(&self) -> SensorResult<(f64, f64, f64)>;
    fn is_connected(&self) -> bool;
    fn source_name(&self) -> &str;
}

pub trait DigitalInputSource: Send + Sync {
    fn read(&self) -> SensorResult<(bool, bool)>;
    fn source_name(&self) -> &str;
    fn is_healthy(&self) -> bool {
        true
    }
}

pub trait SensorSource: Send + Sync {
    fn read_all(&self) -> SensorResult<(f64, f64, f64, bool, bool)>;
    fn is_connected(&self) -> bool;
}

pub trait SensorProvider {
    fn read(&self) -> SensorResult<f64>;
    fn is_connected(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "Unknown Sensor"
    }
}

#[cfg(any(test, feature = "mock"))]
pub use mock::MockSensorSource;

#[cfg(any(test, feature = "mock"))]
mod mock {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub struct MockSensorSource {
        data: Vec<(f64, f64, f64, bool, bool)>,
        current_index: AtomicUsize,
    }

    impl MockSensorSource {
        pub fn new(data: Vec<(f64, f64, f64, bool, bool)>) -> Self {
            Self {
                data,
                current_index: AtomicUsize::new(0),
            }
        }
    }

    impl SensorSource for MockSensorSource {
        fn read_all(&self) -> SensorResult<(f64, f64, f64, bool, bool)> {
            let index = self.current_index.fetch_add(1, Ordering::SeqCst);
            if index < self.data.len() {
                Ok(self.data[index])
            } else {
                Ok(*self.data.last().unwrap_or(&(0.0, 0.0, 0.0, false, false)))
            }
        }

        fn is_connected(&self) -> bool {
            !self.data.is_empty()
        }
    }
}
