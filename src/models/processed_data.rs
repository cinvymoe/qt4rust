// 处理后的数据模型

use std::time::SystemTime;
use super::sensor_data::SensorData;

/// 处理后的数据
#[derive(Debug, Clone)]
pub struct ProcessedData {
    /// 原始传感器数据
    pub raw_data: SensorData,
    
    /// 力矩百分比
    pub moment_percentage: f64,
    
    /// 是否危险
    pub is_danger: bool,
    
    /// 验证错误
    pub validation_error: Option<String>,
    
    /// 时间戳
    pub timestamp: SystemTime,
    
    /// 序列号
    pub sequence_number: u64,
}

impl ProcessedData {
    /// 从传感器数据创建处理后的数据
    pub fn from_sensor_data(raw_data: SensorData, sequence_number: u64) -> Self {
        let moment_percentage = Self::calculate_moment_percentage(&raw_data);
        let is_danger = moment_percentage >= 90.0;
        let validation_error = raw_data.validate().err();
        
        Self {
            raw_data,
            moment_percentage,
            is_danger,
            validation_error,
            timestamp: SystemTime::now(),
            sequence_number,
        }
    }
    
    /// 计算力矩百分比
    fn calculate_moment_percentage(data: &SensorData) -> f64 {
        let current_moment = data.ad1_load * data.ad2_radius;
        let rated_moment = data.rated_load * data.ad2_radius;
        
        if rated_moment > 0.0 {
            (current_moment / rated_moment) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_from_sensor_data() {
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data.clone(), 1);
        
        assert_eq!(processed.sequence_number, 1);
        assert_eq!(processed.raw_data.ad1_load, 20.0);
        assert_eq!(processed.moment_percentage, 80.0);
        assert!(!processed.is_danger);
    }
    
    #[test]
    fn test_danger_detection() {
        let sensor_data = SensorData::new(23.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 2);
        
        assert!(processed.is_danger);
        assert!(processed.moment_percentage >= 90.0);
    }
    
    #[test]
    fn test_validation_error() {
        let mut sensor_data = SensorData::new(20.0, 10.0, 60.0);
        sensor_data.ad1_load = -5.0;  // Invalid negative load
        
        let processed = ProcessedData::from_sensor_data(sensor_data, 3);
        
        assert!(processed.validation_error.is_some());
    }
}
