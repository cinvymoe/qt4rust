// 报警检查器（策略模式）

use crate::models::ProcessedData;
use super::alarm_type::{AlarmType, AlarmLevel, AlarmSource};

/// 报警检查结果
#[derive(Debug, Clone)]
pub struct AlarmCheckResult {
    /// 是否触发报警
    pub triggered: bool,
    /// 报警类型
    pub alarm_type: Option<AlarmType>,
    /// 报警详细信息
    pub message: String,
    /// 报警值（用于记录）
    pub value: f64,
}

impl AlarmCheckResult {
    pub fn no_alarm() -> Self {
        Self {
            triggered: false,
            alarm_type: None,
            message: String::new(),
            value: 0.0,
        }
    }
    
    pub fn alarm(alarm_type: AlarmType, message: String, value: f64) -> Self {
        Self {
            triggered: true,
            alarm_type: Some(alarm_type),
            message,
            value,
        }
    }
}

/// 报警检查器 trait
pub trait AlarmChecker: Send + Sync {
    /// 检查是否触发报警
    fn check(&self, data: &ProcessedData) -> AlarmCheckResult;
    
    /// 获取报警来源
    fn source(&self) -> AlarmSource;
    
    /// 是否启用
    fn is_enabled(&self) -> bool {
        true
    }
}

/// 力矩报警检查器
pub struct MomentAlarmChecker {
    warning_threshold: f64,
    danger_threshold: f64,
    enabled: bool,
}

impl MomentAlarmChecker {
    pub fn new(warning_threshold: f64, danger_threshold: f64) -> Self {
        Self {
            warning_threshold,
            danger_threshold,
            enabled: true,
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl AlarmChecker for MomentAlarmChecker {
    fn check(&self, data: &ProcessedData) -> AlarmCheckResult {
        let percentage = data.moment_percentage;
        
        if percentage >= self.danger_threshold {
            AlarmCheckResult::alarm(
                AlarmType::new(AlarmSource::Moment, AlarmLevel::Danger),
                format!("力矩百分比 {:.1}% 超过危险阈值 {:.1}%", percentage, self.danger_threshold),
                percentage,
            )
        } else if percentage >= self.warning_threshold {
            AlarmCheckResult::alarm(
                AlarmType::new(AlarmSource::Moment, AlarmLevel::Warning),
                format!("力矩百分比 {:.1}% 超过预警阈值 {:.1}%", percentage, self.warning_threshold),
                percentage,
            )
        } else {
            AlarmCheckResult::no_alarm()
        }
    }
    
    fn source(&self) -> AlarmSource {
        AlarmSource::Moment
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// 角度报警检查器
pub struct AngleAlarmChecker {
    min_angle: f64,
    max_angle: f64,
    enabled: bool,
}

impl AngleAlarmChecker {
    pub fn new(min_angle: f64, max_angle: f64) -> Self {
        Self {
            min_angle,
            max_angle,
            enabled: true,
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl AlarmChecker for AngleAlarmChecker {
    fn check(&self, data: &ProcessedData) -> AlarmCheckResult {
        let angle = data.boom_angle;
        
        if angle < self.min_angle {
            AlarmCheckResult::alarm(
                AlarmType::new(AlarmSource::Angle, AlarmLevel::Danger),
                format!("吊臂角度 {:.1}° 低于最小角度 {:.1}°", angle, self.min_angle),
                angle,
            )
        } else if angle > self.max_angle {
            AlarmCheckResult::alarm(
                AlarmType::new(AlarmSource::Angle, AlarmLevel::Danger),
                format!("吊臂角度 {:.1}° 超过最大角度 {:.1}°", angle, self.max_angle),
                angle,
            )
        } else {
            AlarmCheckResult::no_alarm()
        }
    }
    
    fn source(&self) -> AlarmSource {
        AlarmSource::Angle
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// 载荷超限报警检查器
pub struct LoadOverloadChecker {
    max_load: f64,
    enabled: bool,
}

impl LoadOverloadChecker {
    pub fn new(max_load: f64) -> Self {
        Self {
            max_load,
            enabled: true,
        }
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl AlarmChecker for LoadOverloadChecker {
    fn check(&self, data: &ProcessedData) -> AlarmCheckResult {
        let load = data.current_load;
        
        if load > self.max_load {
            AlarmCheckResult::alarm(
                AlarmType::new(AlarmSource::LoadOverload, AlarmLevel::Critical),
                format!("当前载荷 {:.1}t 超过最大载荷 {:.1}t", load, self.max_load),
                load,
            )
        } else {
            AlarmCheckResult::no_alarm()
        }
    }
    
    fn source(&self) -> AlarmSource {
        AlarmSource::LoadOverload
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sensor_data::SensorData;
    
    #[test]
    fn test_moment_alarm_checker() {
        let checker = MomentAlarmChecker::new(90.0, 100.0);
        let sensor_data = SensorData::new(23.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        
        let result = checker.check(&processed);
        assert!(result.triggered);
        assert_eq!(result.alarm_type.unwrap().source, AlarmSource::Moment);
    }
    
    #[test]
    fn test_angle_alarm_checker() {
        let checker = AngleAlarmChecker::new(0.0, 85.0);
        let sensor_data = SensorData::new(20.0, 10.0, 90.0); // 角度超限
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        
        let result = checker.check(&processed);
        assert!(result.triggered);
        assert_eq!(result.alarm_type.unwrap().source, AlarmSource::Angle);
    }
}
