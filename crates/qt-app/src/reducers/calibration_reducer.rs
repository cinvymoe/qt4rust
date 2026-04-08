// 参数校准状态转换器

use crate::states::calibration_state::CalibrationState;
use crate::intents::calibration_intent::CalibrationIntent;

/// 参数校准状态转换器
pub struct CalibrationReducer;

impl CalibrationReducer {
    pub fn new() -> Self {
        Self
    }
    
    /// 状态转换函数（纯函数）
    pub fn reduce(&self, state: CalibrationState, intent: CalibrationIntent) -> CalibrationState {
        match intent {
            CalibrationIntent::ClearError => {
                CalibrationState {
                    error_message: None,
                    ..state
                }
            }
            
            CalibrationIntent::DataUpdated {
                ad1_load,
                ad2_radius,
                ad3_angle,
                calculated_load,
                calculated_radius,
                calculated_angle,
            } => {
                CalibrationState {
                    ad1_load,
                    ad2_radius,
                    ad3_angle,
                    calculated_load,
                    calculated_radius,
                    calculated_angle,
                    sensor_connected: true,
                    error_message: None,
                    last_update_time: std::time::SystemTime::now(),
                }
            }
            
            CalibrationIntent::SensorDisconnected => {
                CalibrationState {
                    sensor_connected: false,
                    error_message: Some("传感器连接断开".to_string()),
                    ..state
                }
            }
            
            CalibrationIntent::SensorReconnected => {
                CalibrationState {
                    sensor_connected: true,
                    error_message: None,
                    ..state
                }
            }
        }
    }
}

impl Default for CalibrationReducer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clear_error() {
        let reducer = CalibrationReducer::new();
        let state = CalibrationState {
            error_message: Some("测试错误".to_string()),
            ..Default::default()
        };
        
        let new_state = reducer.reduce(state, CalibrationIntent::ClearError);
        assert_eq!(new_state.error_message, None);
    }
    
    #[test]
    fn test_data_updated() {
        let reducer = CalibrationReducer::new();
        let state = CalibrationState::default();
        
        let new_state = reducer.reduce(
            state,
            CalibrationIntent::DataUpdated {
                ad1_load: 2047.5,
                ad2_radius: 2047.5,
                ad3_angle: 2047.5,
                calculated_load: 25.0,
                calculated_radius: 10.0,
                calculated_angle: 45.0,
            }
        );
        
        assert_eq!(new_state.ad1_load, 2047.5);
        assert_eq!(new_state.calculated_load, 25.0);
        assert_eq!(new_state.calculated_radius, 10.0);
        assert_eq!(new_state.calculated_angle, 45.0);
        assert!(new_state.sensor_connected);
    }
    
    #[test]
    fn test_sensor_disconnected() {
        let reducer = CalibrationReducer::new();
        let state = CalibrationState {
            sensor_connected: true,
            ..Default::default()
        };
        
        let new_state = reducer.reduce(state, CalibrationIntent::SensorDisconnected);
        assert!(!new_state.sensor_connected);
        assert!(new_state.error_message.is_some());
    }
}
