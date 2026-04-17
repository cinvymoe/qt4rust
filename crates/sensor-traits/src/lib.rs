use std::collections::HashMap;
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

/// 传感器数据结构
#[derive(Debug, Clone, Default)]
pub struct SensorReading {
    /// 模拟传感器值
    pub analog: HashMap<String, f64>,
    /// 数字输入值
    pub digital: HashMap<String, bool>,
}

impl SensorReading {
    pub fn new() -> Self {
        Self {
            analog: HashMap::new(),
            digital: HashMap::new(),
        }
    }

    /// 从元组创建（兼容旧 API）
    pub fn from_tuple(ad1: f64, ad2: f64, ad3: f64, di0: bool, di1: bool) -> Self {
        let mut analog = HashMap::new();
        analog.insert("main_hook_weight".to_string(), ad1);
        analog.insert("radius".to_string(), ad2);
        analog.insert("angle".to_string(), ad3);

        let mut digital = HashMap::new();
        digital.insert("main_hook_switch".to_string(), di0);
        digital.insert("aux_hook_switch".to_string(), di1);

        Self { analog, digital }
    }

    /// 转换为元组（兼容旧 API）
    pub fn to_tuple(&self) -> (f64, f64, f64, bool, bool) {
        (
            self.analog.get("main_hook_weight").copied().unwrap_or(0.0),
            self.analog.get("radius").copied().unwrap_or(0.0),
            self.analog.get("angle").copied().unwrap_or(0.0),
            self.digital
                .get("main_hook_switch")
                .copied()
                .unwrap_or(false),
            self.digital
                .get("aux_hook_switch")
                .copied()
                .unwrap_or(false),
        )
    }
}

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

/// 传感器数据源 trait - 返回 SensorReading
pub trait SensorSource: Send + Sync {
    /// 读取所有传感器数据
    fn read_all(&self) -> SensorResult<SensorReading>;
    /// 检查连接状态
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
        data: Vec<SensorReading>,
        current_index: AtomicUsize,
    }

    impl MockSensorSource {
        pub fn new(data: Vec<SensorReading>) -> Self {
            Self {
                data,
                current_index: AtomicUsize::new(0),
            }
        }

        pub fn from_tuples(data: Vec<(f64, f64, f64, bool, bool)>) -> Self {
            let readings = data
                .into_iter()
                .map(|(a1, a2, a3, d1, d2)| SensorReading::from_tuple(a1, a2, a3, d1, d2))
                .collect();
            Self::new(readings)
        }
    }

    impl SensorSource for MockSensorSource {
        fn read_all(&self) -> SensorResult<SensorReading> {
            let index = self.current_index.fetch_add(1, Ordering::SeqCst);
            if index < self.data.len() {
                Ok(self.data[index].clone())
            } else {
                Ok(self.data.last().cloned().unwrap_or_default())
            }
        }

        fn is_connected(&self) -> bool {
            !self.data.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_reading_from_tuple() {
        let reading = SensorReading::from_tuple(100.0, 50.0, 45.0, true, false);

        assert_eq!(reading.analog.get("main_hook_weight"), Some(&100.0));
        assert_eq!(reading.analog.get("radius"), Some(&50.0));
        assert_eq!(reading.analog.get("angle"), Some(&45.0));
        assert_eq!(reading.digital.get("main_hook_switch"), Some(&true));
        assert_eq!(reading.digital.get("aux_hook_switch"), Some(&false));
    }

    #[test]
    fn test_sensor_reading_to_tuple() {
        let reading = SensorReading::from_tuple(100.0, 50.0, 45.0, true, false);
        let (a1, a2, a3, d1, d2) = reading.to_tuple();

        assert_eq!(a1, 100.0);
        assert_eq!(a2, 50.0);
        assert_eq!(a3, 45.0);
        assert_eq!(d1, true);
        assert_eq!(d2, false);
    }

    #[test]
    fn test_mock_sensor_source() {
        let mock = MockSensorSource::from_tuples(vec![
            (100.0, 50.0, 45.0, false, false),
            (200.0, 60.0, 50.0, true, true),
        ]);

        let reading1 = mock.read_all().unwrap();
        assert_eq!(reading1.analog.get("main_hook_weight"), Some(&100.0));

        let reading2 = mock.read_all().unwrap();
        assert_eq!(reading2.analog.get("main_hook_weight"), Some(&200.0));
    }
}
