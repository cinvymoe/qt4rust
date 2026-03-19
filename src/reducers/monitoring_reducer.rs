// 监控状态转换器

use crate::states::monitoring_state::MonitoringState;
use crate::intents::MonitoringIntent;
use crate::models::SensorData;

/// 监控状态转换器
pub struct MonitoringReducer;

impl MonitoringReducer {
    pub fn new() -> Self {
        Self
    }
    
    /// 状态转换函数（纯函数）
    pub fn reduce(&self, state: MonitoringState, intent: MonitoringIntent) -> MonitoringState {
        match intent {
            MonitoringIntent::ClearError => {
                MonitoringState {
                    error_message: None,
                    ..state
                }
            }
            
            MonitoringIntent::ResetAlarm => {
                MonitoringState {
                    is_danger: false,
                    ..state
                }
            }
            
            MonitoringIntent::SensorDataUpdated(sensor_data) => {
                self.update_from_sensor_data(state, sensor_data)
            }
            
            MonitoringIntent::SensorDisconnected => {
                MonitoringState {
                    sensor_connected: false,
                    error_message: Some("传感器连接断开".to_string()),
                    ..state
                }
            }
            
            MonitoringIntent::SensorReconnected => {
                MonitoringState {
                    sensor_connected: true,
                    error_message: None,
                    ..state
                }
            }
        }
    }
    
    /// 从传感器数据更新状态
    fn update_from_sensor_data(&self, _state: MonitoringState, sensor_data: SensorData) -> MonitoringState {
        // 计算力矩百分比
        let moment_percentage = self.calculate_moment_percentage(&sensor_data);
        
        // 判断是否危险
        let is_danger = moment_percentage >= 90.0;
        
        // 数据验证
        let error_message = match sensor_data.validate() {
            Ok(_) => None,
            Err(e) => Some(e),
        };
        
        MonitoringState {
            current_load: sensor_data.ad1_load,
            rated_load: sensor_data.rated_load,
            working_radius: sensor_data.ad2_radius,
            boom_angle: sensor_data.ad3_angle,
            boom_length: sensor_data.boom_length,
            moment_percentage,
            is_danger,
            sensor_connected: true,
            error_message,
            last_update_time: std::time::SystemTime::now(),
        }
    }
    
    /// 计算力矩百分比
    fn calculate_moment_percentage(&self, sensor_data: &SensorData) -> f64 {
        sensor_data.calculate_moment_percentage()
    }
}

impl Default for MonitoringReducer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clear_error() {
        let reducer = MonitoringReducer::new();
        let state = MonitoringState {
            error_message: Some("测试错误".to_string()),
            ..Default::default()
        };
        
        let new_state = reducer.reduce(state, MonitoringIntent::ClearError);
        assert_eq!(new_state.error_message, None);
    }
    
    #[test]
    fn test_sensor_data_updated() {
        let reducer = MonitoringReducer::new();
        let state = MonitoringState::default();
        let sensor_data = SensorData::new(17.0, 10.0, 62.7);
        
        let new_state = reducer.reduce(
            state,
            MonitoringIntent::SensorDataUpdated(sensor_data)
        );
        
        assert_eq!(new_state.current_load, 17.0);
        assert_eq!(new_state.working_radius, 10.0);
        assert!(new_state.sensor_connected);
    }
    
    #[test]
    fn test_reset_alarm() {
        let reducer = MonitoringReducer::new();
        let state = MonitoringState {
            is_danger: true,
            ..Default::default()
        };
        
        let new_state = reducer.reduce(state, MonitoringIntent::ResetAlarm);
        assert!(!new_state.is_danger);
    }
}
