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
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;
use crate::states::MonitoringState;
use crate::intents::MonitoringIntent;
use crate::reducers::MonitoringReducer;
use cxx_qt_mvi_core::prelude::Reducer;
use std::time::Instant;
use cxx_qt::CxxQtType;

/// MonitoringViewModel 实现
pub struct MonitoringViewModelRust {
    /// 当前状态
    state: MonitoringState,
    
    /// 状态转换器
    reducer: MonitoringReducer,
    
    /// 最后更新时间（用于限制刷新频率）
    last_update_time: Instant,
    
    /// 最小更新间隔（毫秒）
    min_update_interval_ms: u64,
    
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
            state,
            reducer: MonitoringReducer::new(),
            last_update_time: Instant::now(),
            min_update_interval_ms: 100,  // 最小 100ms 更新间隔，即最高 10Hz 刷新率
        }
    }
}

impl monitoring_viewmodel_bridge::MonitoringViewModel {
    /// 处理意图（公开方法，供后台线程调用）
    pub fn handle_intent(mut self: Pin<&mut Self>, intent: MonitoringIntent) {
        // 对于传感器数据更新，检查刷新频率限制（节流）
        if matches!(intent, MonitoringIntent::SensorDataUpdated(_)) {
            let now = Instant::now();
            // 获取rust引用并存储在局部变量中以延长生命周期
            let binding = self.as_ref();
            let rust_ref = binding.rust();
            let elapsed = now.duration_since(rust_ref.last_update_time).as_millis() as u64;
            
            // 如果距离上次更新时间太短，跳过本次更新
            if elapsed < rust_ref.min_update_interval_ms {
                return;
            }
            
            // 更新最后更新时间
            let mut_binding = self.as_mut();
            let mut mut_rust_ref = mut_binding.rust_mut();
            mut_rust_ref.last_update_time = now;
        }
        
        // 1. 调用 Reducer 计算新状态
        let binding = self.as_ref();
        let rust_ref = binding.rust();
        let new_state = rust_ref.reducer.reduce(rust_ref.state.clone(), intent);
        
        // 2. 更新状态
        self.update_state(new_state);
    }
    
    /// 更新状态并触发 Qt 属性变化信号
    fn update_state(mut self: Pin<&mut Self>, new_state: MonitoringState) {
        // 获取旧状态的副本用于比较
        let binding = self.as_ref();
        let rust_ref = binding.rust();
        let old_state = rust_ref.state.clone();
        
        // 只更新变化的属性，避免不必要的 UI 刷新
        if old_state.current_load != new_state.current_load {
            self.as_mut().set_current_load(new_state.current_load);
        }
        if old_state.rated_load != new_state.rated_load {
            self.as_mut().set_rated_load(new_state.rated_load);
        }
        if old_state.working_radius != new_state.working_radius {
            self.as_mut().set_working_radius(new_state.working_radius);
        }
        if old_state.boom_angle != new_state.boom_angle {
            self.as_mut().set_boom_angle(new_state.boom_angle);
        }
        if old_state.boom_length != new_state.boom_length {
            self.as_mut().set_boom_length(new_state.boom_length);
        }
        if old_state.moment_percentage != new_state.moment_percentage {
            self.as_mut().set_moment_percentage(new_state.moment_percentage);
        }
        if old_state.is_danger != new_state.is_danger {
            self.as_mut().set_is_danger(new_state.is_danger);
        }
        if old_state.sensor_connected != new_state.sensor_connected {
            self.as_mut().set_sensor_connected(new_state.sensor_connected);
        }
        if old_state.error_message != new_state.error_message {
            let error_msg = new_state.error_message.clone().unwrap_or_default();
            self.as_mut().set_error_message(QString::from(&error_msg));
        }
        
        // 更新内部状态
        let mut_binding = self.as_mut();
        let mut mut_rust_ref = mut_binding.rust_mut();
        mut_rust_ref.state = new_state;
    }
    
    /// 清除错误
    pub fn clear_error(mut self: Pin<&mut Self>) {
        self.as_mut().handle_intent(MonitoringIntent::ClearError);
    }
    
    /// 重置报警
    pub fn reset_alarm(mut self: Pin<&mut Self>) {
        self.as_mut().handle_intent(MonitoringIntent::ResetAlarm);
    }
}
