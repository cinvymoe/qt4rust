// 滤波缓冲区 - 多速率数据流架构
// 采集(10ms) → 滤波层 → 计算(100ms) → 存储(500ms)

use crate::models::sensor_data::SensorData;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    None,
    Mean,
    Median,
}

impl Default for FilterType {
    fn default() -> Self {
        FilterType::Mean
    }
}

impl std::fmt::Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterType::None => write!(f, "None"),
            FilterType::Mean => write!(f, "Mean"),
            FilterType::Median => write!(f, "Median"),
        }
    }
}

impl std::str::FromStr for FilterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(FilterType::None),
            "mean" => Ok(FilterType::Mean),
            "median" => Ok(FilterType::Median),
            _ => Err(format!("Unknown filter type: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterBufferConfig {
    pub filter_type: FilterType,
    pub window_size: usize,
}

impl Default for FilterBufferConfig {
    fn default() -> Self {
        Self {
            filter_type: FilterType::Mean,
            window_size: 10,
        }
    }
}

impl FilterBufferConfig {
    pub fn from_str(filter_type: &str, window_size: usize) -> Result<Self, String> {
        Ok(Self {
            filter_type: filter_type.parse()?,
            window_size,
        })
    }
}

#[derive(Debug)]
pub struct FilterBuffer {
    raw_data: VecDeque<SensorData>,
    config: FilterBufferConfig,
}

impl FilterBuffer {
    pub fn new(config: FilterBufferConfig) -> Self {
        Self {
            raw_data: VecDeque::with_capacity(config.window_size * 2),
            config,
        }
    }

    pub fn push(&mut self, data: SensorData) {
        self.raw_data.push_back(data);
        // Warn if approaching overflow (get_filtered not called fast enough)
        if self.raw_data.len() > self.config.window_size * 2 {
            tracing::warn!(
                "FilterBuffer overflow: {} items (window_size={}). \
                 Process pipeline may be stalled.",
                self.raw_data.len(),
                self.config.window_size
            );
        }
        while self.raw_data.len() > self.config.window_size * 2 {
            self.raw_data.pop_front();
        }
    }

    pub fn get_filtered(&self) -> Option<SensorData> {
        if self.raw_data.is_empty() {
            return None;
        }
        if self.raw_data.len() < self.config.window_size {
            return self.raw_data.back().cloned();
        }

        let window: Vec<SensorData> = self
            .raw_data
            .iter()
            .rev()
            .take(self.config.window_size)
            .cloned()
            .collect();

        match self.config.filter_type {
            FilterType::None => window.into_iter().last(),
            FilterType::Mean => Some(Self::mean_filter(&window)),
            FilterType::Median => Some(Self::median_filter(&window)),
        }
    }

    pub fn len(&self) -> usize {
        self.raw_data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.raw_data.is_empty()
    }
    pub fn is_ready(&self) -> bool {
        self.raw_data.len() >= self.config.window_size
    }
    pub fn clear(&mut self) {
        self.raw_data.clear();
    }
    pub fn config(&self) -> &FilterBufferConfig {
        &self.config
    }
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.config.filter_type = filter_type;
    }

    fn mean_filter(data: &[SensorData]) -> SensorData {
        if data.is_empty() {
            return SensorData::new(0.0, 0.0, 0.0);
        }
        let count = data.len() as f64;
        let (a, r, g) = data.iter().fold((0.0, 0.0, 0.0), |(a, r, g), d| {
            (a + d.ad1_load, r + d.ad2_radius, g + d.ad3_angle)
        });
        SensorData::new(a / count, r / count, g / count)
    }

    fn median_filter(data: &[SensorData]) -> SensorData {
        if data.is_empty() {
            return SensorData::new(0.0, 0.0, 0.0);
        }
        let mut ad1: Vec<f64> = data.iter().map(|d| d.ad1_load).collect();
        let mut ad2: Vec<f64> = data.iter().map(|d| d.ad2_radius).collect();
        let mut ad3: Vec<f64> = data.iter().map(|d| d.ad3_angle).collect();
        ad1.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ad2.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ad3.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = data.len() / 2;
        SensorData::new(ad1[mid], ad2[mid], ad3[mid])
    }
}

impl Default for FilterBuffer {
    fn default() -> Self {
        Self::new(FilterBufferConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sd(ad1: f64, ad2: f64, ad3: f64) -> SensorData {
        SensorData::new(ad1, ad2, ad3)
    }

    #[test]
    fn test_none_filter() {
        let mut buffer = FilterBuffer::new(FilterBufferConfig {
            filter_type: FilterType::None,
            window_size: 5,
        });
        buffer.push(sd(10.0, 5.0, 30.0));
        buffer.push(sd(20.0, 6.0, 31.0));
        buffer.push(sd(30.0, 7.0, 32.0));
        let r = buffer.get_filtered().unwrap();
        assert_eq!(r.ad1_load, 30.0);
    }

    #[test]
    fn test_mean_filter() {
        let mut buffer = FilterBuffer::new(FilterBufferConfig {
            filter_type: FilterType::Mean,
            window_size: 5,
        });
        buffer.push(sd(10.0, 5.0, 30.0));
        buffer.push(sd(20.0, 6.0, 31.0));
        buffer.push(sd(30.0, 7.0, 32.0));
        buffer.push(sd(40.0, 8.0, 33.0));
        buffer.push(sd(50.0, 9.0, 34.0));
        let r = buffer.get_filtered().unwrap();
        assert_eq!(r.ad1_load, 30.0);
        assert_eq!(r.ad2_radius, 7.0);
    }

    #[test]
    fn test_median_filter() {
        let mut buffer = FilterBuffer::new(FilterBufferConfig {
            filter_type: FilterType::Median,
            window_size: 5,
        });
        buffer.push(sd(10.0, 5.0, 30.0));
        buffer.push(sd(30.0, 7.0, 32.0));
        buffer.push(sd(20.0, 6.0, 31.0));
        buffer.push(sd(50.0, 9.0, 34.0));
        buffer.push(sd(40.0, 8.0, 33.0));
        let r = buffer.get_filtered().unwrap();
        assert_eq!(r.ad1_load, 30.0);
    }

    #[test]
    fn test_window_overflow() {
        let mut buffer = FilterBuffer::new(FilterBufferConfig {
            filter_type: FilterType::Mean,
            window_size: 3,
        });
        for i in 0..5 {
            buffer.push(sd((i + 1) as f64 * 10.0, 5.0, 30.0));
        }
        let r = buffer.get_filtered().unwrap();
        assert_eq!(r.ad1_load, 40.0);
    }

    #[test]
    fn test_is_ready() {
        let mut buffer = FilterBuffer::new(FilterBufferConfig {
            filter_type: FilterType::Mean,
            window_size: 3,
        });
        assert!(!buffer.is_ready());
        buffer.push(sd(10.0, 5.0, 30.0));
        buffer.push(sd(20.0, 6.0, 31.0));
        buffer.push(sd(30.0, 7.0, 32.0));
        assert!(buffer.is_ready());
    }

    #[test]
    fn test_filter_type_from_str() {
        assert_eq!("mean".parse::<FilterType>().unwrap(), FilterType::Mean);
        assert!("invalid".parse::<FilterType>().is_err());
    }

    #[test]
    fn test_empty() {
        let buffer = FilterBuffer::new(FilterBufferConfig::default());
        assert!(buffer.is_empty());
        assert!(buffer.get_filtered().is_none());
    }
}
