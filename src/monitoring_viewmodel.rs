// 监控视图 ViewModel

#[cxx_qt::bridge]
pub mod monitoring_viewmodel_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, current_load)]
        #[qproperty(f64, rated_load)]
        #[qproperty(f64, working_radius)]
        #[qproperty(f64, boom_angle)]
        #[qproperty(f64, boom_length)]
        #[qproperty(f64, moment_percentage)]
        #[qproperty(bool, is_danger)]
        #[qproperty(bool, sensor_connected)]
        #[qproperty(QString, error_message)]
        type MonitoringViewModel = super::MonitoringViewModelRust;

        /// 清除错误（Intent）
        #[qinvokable]
        unsafe fn clear_error(self: Pin<&mut MonitoringViewModel>);

        /// 重置报警（Intent）
        #[qinvokable]
        unsafe fn reset_alarm(self: Pin<&mut MonitoringViewModel>);
        
        /// 测试方法：更新模拟数据（仅用于开发测试）
        #[qinvokable]
        unsafe fn update_test_data(
            self: Pin<&mut MonitoringViewModel>,
            load: f64,
            radius: f64,
            angle: f64
        );
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;
use crate::states::monitoring_state::MonitoringState;
use crate::intents::monitoring_intent::MonitoringIntent;
use crate::reducers::monitoring_reducer::MonitoringReducer;

/// MonitoringViewModel 实现
pub struct MonitoringViewModelRust {
    // Qt 属性字段
    current_load: f64,
    rated_load: f64,
    working_radius: f64,
    boom_angle: f64,
    boom_length: f64,
    moment_percentage: f64,
    is_danger: bool,
    sensor_connected: bool,
    error_message: QString,
    
    // 内部状态（不暴露给 QML）
    reducer: MonitoringReducer,
}

impl Default for MonitoringViewModelRust {
    fn default() -> Self {
        let state = MonitoringState::default();
        Self {
            current_load: state.current_load,
            rated_load: state.rated_load,
            working_radius: state.working_radius,
            boom_angle: state.boom_angle,
            boom_length: state.boom_length,
            moment_percentage: state.moment_percentage,
            is_danger: state.is_danger,
            sensor_connected: state.sensor_connected,
            error_message: QString::from(""),
            reducer: MonitoringReducer::new(),
        }
    }
}

impl monitoring_viewmodel_bridge::MonitoringViewModel {
    /// 处理意图（公开方法，供后台线程调用）
    pub fn handle_intent(self: Pin<&mut Self>, intent: MonitoringIntent) {
        // 构建当前状态
        let current_state = MonitoringState {
            current_load: *self.as_ref().current_load(),
            rated_load: *self.as_ref().rated_load(),
            working_radius: *self.as_ref().working_radius(),
            boom_angle: *self.as_ref().boom_angle(),
            boom_length: *self.as_ref().boom_length(),
            moment_percentage: *self.as_ref().moment_percentage(),
            is_danger: *self.as_ref().is_danger(),
            sensor_connected: *self.as_ref().sensor_connected(),
            error_message: {
                let msg = self.as_ref().error_message().to_string();
                if msg.is_empty() { None } else { Some(msg) }
            },
            last_update_time: std::time::SystemTime::now(),
        };
        
        // 调用 Reducer 计算新状态
        let new_state = self.reducer.reduce(current_state, intent);
        
        // 更新状态
        self.update_state(new_state);
    }
    
    /// 更新状态并触发 Qt 属性变化信号
    fn update_state(mut self: Pin<&mut Self>, new_state: MonitoringState) {
        // 只更新变化的属性，避免不必要的 UI 刷新
        if *self.as_ref().current_load() != new_state.current_load {
            self.as_mut().set_current_load(new_state.current_load);
        }
        if *self.as_ref().rated_load() != new_state.rated_load {
            self.as_mut().set_rated_load(new_state.rated_load);
        }
        if *self.as_ref().working_radius() != new_state.working_radius {
            self.as_mut().set_working_radius(new_state.working_radius);
        }
        if *self.as_ref().boom_angle() != new_state.boom_angle {
            self.as_mut().set_boom_angle(new_state.boom_angle);
        }
        if *self.as_ref().boom_length() != new_state.boom_length {
            self.as_mut().set_boom_length(new_state.boom_length);
        }
        if *self.as_ref().moment_percentage() != new_state.moment_percentage {
            self.as_mut().set_moment_percentage(new_state.moment_percentage);
        }
        if *self.as_ref().is_danger() != new_state.is_danger {
            self.as_mut().set_is_danger(new_state.is_danger);
        }
        if *self.as_ref().sensor_connected() != new_state.sensor_connected {
            self.as_mut().set_sensor_connected(new_state.sensor_connected);
        }
        
        let current_error = self.as_ref().error_message().to_string();
        let new_error = new_state.error_message.clone().unwrap_or_default();
        if current_error != new_error {
            self.as_mut().set_error_message(QString::from(&new_error));
        }
    }
    
    /// 清除错误
    pub fn clear_error(mut self: Pin<&mut Self>) {
        self.as_mut().handle_intent(MonitoringIntent::ClearError);
    }
    
    /// 重置报警
    pub fn reset_alarm(mut self: Pin<&mut Self>) {
        self.as_mut().handle_intent(MonitoringIntent::ResetAlarm);
    }
    
    /// 测试方法：更新模拟数据（仅用于开发测试）
    pub fn update_test_data(mut self: Pin<&mut Self>, load: f64, radius: f64, angle: f64) {
        // 创建模拟的传感器数据
        let sensor_data = crate::models::SensorData::new(load, radius, angle);
        
        // 通过 Intent 更新状态
        self.as_mut().handle_intent(MonitoringIntent::SensorDataUpdated(sensor_data));
    }
}
