// 报警记录模型

use super::processed_data::ProcessedData;
use std::time::SystemTime;

/// 报警类型
#[derive(Debug, Clone, PartialEq)]
pub enum AlarmType {
    /// 预警（90-100%）
    Warning,
    /// 危险（>100%）
    Danger,
}

impl AlarmType {
    pub fn as_str(&self) -> &str {
        match self {
            AlarmType::Warning => "warning",
            AlarmType::Danger => "danger",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "warning" => Some(AlarmType::Warning),
            "danger" => Some(AlarmType::Danger),
            _ => None,
        }
    }
}

/// 报警记录
#[derive(Debug, Clone)]
pub struct AlarmRecord {
    /// 报警 ID（数据库主键）
    pub id: Option<i64>,

    /// 关联的数据序列号
    pub sequence_number: u64,

    /// 报警时间戳
    pub timestamp: SystemTime,

    /// 报警类型
    pub alarm_type: AlarmType,

    /// 当前载荷（吨）
    pub current_load: f64,

    /// 额定载荷（吨）
    pub rated_load: f64,

    /// 工作半径（米）
    pub working_radius: f64,

    /// 吊臂角度（度）
    pub boom_angle: f64,

    /// 臂长（米）
    pub boom_length: f64,

    /// 力矩百分比
    pub moment_percentage: f64,

    /// 报警描述
    pub description: String,

    /// 是否已确认
    pub acknowledged: bool,

    /// 确认时间
    pub acknowledged_at: Option<SystemTime>,
}

impl AlarmRecord {
    /// 从处理后的数据创建报警记录
    pub fn from_processed_data(data: &ProcessedData) -> Self {
        let alarm_type = if data.moment_percentage >= 100.0 {
            AlarmType::Danger
        } else {
            AlarmType::Warning
        };

        let description = format!(
            "力矩百分比 {:.1}% 超过阈值，当前载荷 {:.1}t，工作半径 {:.1}m",
            data.moment_percentage, data.current_load, data.working_radius
        );

        Self {
            id: None,
            sequence_number: data.sequence_number,
            timestamp: data.timestamp,
            alarm_type,
            current_load: data.current_load,
            rated_load: 25.0, // TODO: 从配置获取
            working_radius: data.working_radius,
            boom_angle: data.boom_angle,
            boom_length: 0.0, // TODO: 从传感器数据获取
            moment_percentage: data.moment_percentage,
            description,
            acknowledged: false,
            acknowledged_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sensor_core::SensorData;

    #[test]
    fn test_alarm_type_as_str() {
        assert_eq!(AlarmType::Warning.as_str(), "warning");
        assert_eq!(AlarmType::Danger.as_str(), "danger");
    }

    #[test]
    fn test_alarm_type_from_str() {
        assert_eq!(AlarmType::from_str("warning"), Some(AlarmType::Warning));
        assert_eq!(AlarmType::from_str("danger"), Some(AlarmType::Danger));
        assert_eq!(AlarmType::from_str("invalid"), None);
    }

    #[test]
    fn test_from_processed_data_warning() {
        let sensor_data = SensorData::new(23.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        let alarm = AlarmRecord::from_processed_data(&processed);

        assert_eq!(alarm.sequence_number, 1);
        assert_eq!(alarm.alarm_type, AlarmType::Warning);
        assert!(!alarm.acknowledged);
        assert!(alarm.description.contains("力矩百分比"));
    }

    #[test]
    fn test_from_processed_data_danger() {
        let sensor_data = SensorData::new(26.0, 10.0, 60.0, false, false);
        let processed = ProcessedData::from_sensor_data(sensor_data, 2);
        let alarm = AlarmRecord::from_processed_data(&processed);

        assert_eq!(alarm.alarm_type, AlarmType::Danger);
        assert!(alarm.moment_percentage >= 100.0);
    }
}
