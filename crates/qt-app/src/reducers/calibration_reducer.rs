// 参数校准状态转换器

use crate::states::calibration_state::CalibrationState;
use crate::intents::calibration_intent::CalibrationIntent;
use qt_rust_demo::models::SensorData;

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
            
            CalibrationIntent::SensorDataUpdated(sensor_data) => {
                self.update_from_sensor_data(state, sensor_data)
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
    
    /// 从传感器数据更新状态
    fn update_from_sensor_data(&self, _state: CalibrationState, sensor_data: SensorData) -> CalibrationState {
        // 数据验证
        let error_message = match sensor_data.validate() {
            Ok(_) => None,
            Err(e) => Some(e),
        };
        
        CalibrationState {
            ad1_load: sensor_data.ad1_load,
            ad2_radius: sensor_data.ad2_radius,
            ad3_angle: sensor_data.ad3_angle,
            sensor_connected: true,
            error_message,
            last_update_time: std::time::SystemTime::now(),
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
    fn test_sensor_data_updated() {
        let reducer = CalibrationReducer::new();
        let state = CalibrationState::default();
        let sensor_data = SensorData::new(17.0, 10.0, 62.7);
        
        let new_state = reducer.reduce(
            state,
            CalibrationIntent::SensorDataUpdated(sensor_data)
        );
        
        assert_eq!(new_state.ad1_load, 17.0);
        assert_eq!(new_state.ad2_radius, 10.0);
        assert_eq!(new_state.ad3_angle, 62.7);
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
