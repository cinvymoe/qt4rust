// 监控视图状态

use std::time::SystemTime;

/// 监控视图状态
#[derive(Debug, Clone, PartialEq)]
pub struct MonitoringState {
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
    
    /// 是否处于危险状态
    pub is_danger: bool,
    
    /// 传感器连接状态
    pub sensor_connected: bool,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 最后更新时间
    pub last_update_time: SystemTime,
}

impl Default for MonitoringState {
    fn default() -> Self {
        Self {
            current_load: 0.0,
            rated_load: 25.0,
            working_radius: 0.0,
            boom_angle: 0.0,
            boom_length: 0.0,
            moment_percentage: 0.0,
            is_danger: false,
            sensor_connected: false,
            error_message: None,
            last_update_time: SystemTime::now(),
        }
    }
}

impl MonitoringState {
    /// 创建新的监控状态
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 检查是否需要报警
    pub fn should_alarm(&self) -> bool {
        self.moment_percentage >= 90.0
    }
    
    /// 获取报警级别 (0: 正常, 1: 预警, 2: 危险)
    pub fn alarm_level(&self) -> u8 {
        if self.moment_percentage >= 100.0 {
            2
        } else if self.moment_percentage >= 90.0 {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_state() {
        let state = MonitoringState::default();
        assert_eq!(state.current_load, 0.0);
        assert_eq!(state.rated_load, 25.0);
        assert!(!state.is_danger);
        assert!(!state.sensor_connected);
    }
    
    #[test]
    fn test_should_alarm() {
        let mut state = MonitoringState::default();
        
        state.moment_percentage = 85.0;
        assert!(!state.should_alarm());
        
        state.moment_percentage = 90.0;
        assert!(state.should_alarm());
        
        state.moment_percentage = 105.0;
        assert!(state.should_alarm());
    }
    
    #[test]
    fn test_alarm_level() {
        let mut state = MonitoringState::default();
        
        state.moment_percentage = 50.0;
        assert_eq!(state.alarm_level(), 0);
        
        state.moment_percentage = 95.0;
        assert_eq!(state.alarm_level(), 1);
        
        state.moment_percentage = 110.0;
        assert_eq!(state.alarm_level(), 2);
    }
}
