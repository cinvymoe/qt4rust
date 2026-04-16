// 监控状态转换器

use crate::intents::MonitoringIntent;
use crate::models::ProcessedData;
use crate::states::monitoring_state::MonitoringState;

/// 监控状态转换器
///
/// 注意：此 Reducer 不再处理原始传感器数据，而是直接使用
/// 从共享管道（SharedBuffer）中获取的已处理数据（ProcessedData）
pub struct MonitoringReducer;

impl MonitoringReducer {
    pub fn new() -> Self {
        Self
    }

    /// 状态转换函数（纯函数）
    pub fn reduce(&self, state: MonitoringState, intent: MonitoringIntent) -> MonitoringState {
        match intent {
            MonitoringIntent::ClearError => MonitoringState {
                error_message: None,
                ..state
            },

            MonitoringIntent::ResetAlarm => MonitoringState {
                is_danger: false,
                ..state
            },

            MonitoringIntent::ProcessedDataUpdated(processed_data) => {
                self.update_from_processed_data(state, processed_data)
            }

            MonitoringIntent::SensorDisconnected => MonitoringState {
                sensor_connected: false,
                error_message: Some("传感器连接断开".to_string()),
                ..state
            },

            MonitoringIntent::SensorReconnected => MonitoringState {
                sensor_connected: true,
                error_message: None,
                ..state
            },
        }
    }

    /// 从已处理数据更新状态
    ///
    /// ProcessedData 已经包含了：
    /// - AD值转换后的物理量（重量、角度、半径）
    /// - 计算后的力矩百分比
    /// - 危险状态判断
    /// - 数据验证结果
    fn update_from_processed_data(
        &self,
        _state: MonitoringState,
        processed: ProcessedData,
    ) -> MonitoringState {
        tracing::info!("[MonitoringReducer] 更新状态: load={:.2}吨, rated={:.2}吨, radius={:.2}米, angle={:.2}度, moment={:.1}%",
            processed.current_load, processed.rated_load, processed.working_radius, processed.boom_angle, processed.moment_percentage);

        MonitoringState {
            current_load: processed.current_load,     // 转换后的重量值（吨）
            rated_load: processed.rated_load,         // 从载荷表查询得到的额定载荷
            working_radius: processed.working_radius, // 转换后的工作半径（米）
            boom_angle: processed.boom_angle,         // 转换后的角度（度）
            boom_length: processed.boom_length,       // 从处理后的数据获取臂长
            moment_percentage: processed.moment_percentage, // 计算后的力矩百分比
            is_warning: processed.is_warning,         // 预警状态判断
            is_danger: processed.is_danger,           // 危险状态判断
            sensor_connected: true,
            error_message: processed.validation_error, // 验证错误信息
            last_update_time: processed.timestamp,
        }
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
    use crate::models::SensorData;

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
    fn test_processed_data_updated() {
        let reducer = MonitoringReducer::new();
        let state = MonitoringState::default();

        // 创建已处理的数据
        let sensor_data = SensorData::new(2047.5, 2047.5, 2047.5, false, false);
        let processed_data = ProcessedData::from_sensor_data(sensor_data, 1);

        let new_state = reducer.reduce(
            state,
            MonitoringIntent::ProcessedDataUpdated(processed_data),
        );

        // 验证使用了处理后的值
        assert!(new_state.current_load > 0.0);
        assert!(new_state.working_radius > 0.0);
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
