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
        #[qproperty(f64, raw_ad1_load)]
        #[qproperty(f64, raw_ad2_radius)]
        #[qproperty(f64, raw_ad3_angle)]
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
            angle: f64,
        );

        /// 手动触发显示更新（供 QML Timer 调用）
        #[qinvokable]
        unsafe fn tick_display(self: Pin<&mut MonitoringViewModel>) -> bool;

        /// 初始化显示管道（从全局共享缓冲区）
        #[qinvokable]
        unsafe fn init_display_pipeline_from_global(self: Pin<&mut MonitoringViewModel>);

        /// 从全局传感器缓冲区更新原始传感器数据
        #[qinvokable]
        unsafe fn update_raw_sensor_data(self: Pin<&mut MonitoringViewModel>);
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;
use qt_rust_demo::intents::monitoring_intent::MonitoringIntent;
use qt_rust_demo::pipeline::shared_buffer::SharedBuffer;
use qt_rust_demo::pipeline::DisplayPipeline;
use qt_rust_demo::reducers::monitoring_reducer::MonitoringReducer;
use qt_rust_demo::states::monitoring_state::MonitoringState;

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
    display_pipeline: std::cell::RefCell<Option<Box<DisplayPipeline>>>,

    raw_ad1_load: f64,
    raw_ad2_radius: f64,
    raw_ad3_angle: f64,
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
            display_pipeline: std::cell::RefCell::new(None),
            raw_ad1_load: 0.0,
            raw_ad2_radius: 0.0,
            raw_ad3_angle: 0.0,
        }
    }
}

impl monitoring_viewmodel_bridge::MonitoringViewModel {
    /// 处理意图（公开方法，供后台线程调用）
    pub fn handle_intent(self: Pin<&mut Self>, intent: MonitoringIntent) {
        // 打印 Intent 信息
        tracing::debug!("[MonitoringViewModel] Handling intent: {:?}", intent);

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
                if msg.is_empty() {
                    None
                } else {
                    Some(msg)
                }
            },
            last_update_time: std::time::SystemTime::now(),
        };

        // 调用 Reducer 计算新状态
        tracing::trace!("[MonitoringViewModel] Current state before reduce: current_load={:.2}, radius={:.2}, angle={:.2}, moment={:.2}%", 
            current_state.current_load, current_state.working_radius, current_state.boom_angle, current_state.moment_percentage);

        let new_state = self.reducer.reduce(current_state, intent);

        tracing::trace!("[MonitoringViewModel] New state after reduce: current_load={:.2}, radius={:.2}, angle={:.2}, moment={:.2}%", 
            new_state.current_load, new_state.working_radius, new_state.boom_angle, new_state.moment_percentage);

        // 更新状态
        self.update_state(new_state);
    }

    /// 更新状态并触发 Qt 属性变化信号
    fn update_state(mut self: Pin<&mut Self>, new_state: MonitoringState) {
        tracing::trace!("[MonitoringViewModel] Updating state...");

        // 只更新变化的属性，避免不必要的 UI 刷新
        if *self.as_ref().current_load() != new_state.current_load {
            tracing::trace!(
                "[MonitoringViewModel] Updating current_load: {:.2} -> {:.2}",
                *self.as_ref().current_load(),
                new_state.current_load
            );
            self.as_mut().set_current_load(new_state.current_load);
        }
        if *self.as_ref().rated_load() != new_state.rated_load {
            self.as_mut().set_rated_load(new_state.rated_load);
        }
        if *self.as_ref().working_radius() != new_state.working_radius {
            tracing::trace!(
                "[MonitoringViewModel] Updating working_radius: {:.2} -> {:.2}",
                *self.as_ref().working_radius(),
                new_state.working_radius
            );
            self.as_mut().set_working_radius(new_state.working_radius);
        }
        if *self.as_ref().boom_angle() != new_state.boom_angle {
            tracing::trace!(
                "[MonitoringViewModel] Updating boom_angle: {:.2} -> {:.2}",
                *self.as_ref().boom_angle(),
                new_state.boom_angle
            );
            self.as_mut().set_boom_angle(new_state.boom_angle);
        }
        if *self.as_ref().boom_length() != new_state.boom_length {
            self.as_mut().set_boom_length(new_state.boom_length);
        }
        if *self.as_ref().moment_percentage() != new_state.moment_percentage {
            tracing::trace!(
                "[MonitoringViewModel] Updating moment_percentage: {:.2}% -> {:.2}%",
                *self.as_ref().moment_percentage(),
                new_state.moment_percentage
            );
            self.as_mut()
                .set_moment_percentage(new_state.moment_percentage);
        }
        if *self.as_ref().is_danger() != new_state.is_danger {
            tracing::info!(
                "[MonitoringViewModel] Danger state changed: {} -> {}",
                *self.as_ref().is_danger(),
                new_state.is_danger
            );
            self.as_mut().set_is_danger(new_state.is_danger);
        }
        if *self.as_ref().sensor_connected() != new_state.sensor_connected {
            tracing::info!(
                "[MonitoringViewModel] Sensor connection changed: {} -> {}",
                *self.as_ref().sensor_connected(),
                new_state.sensor_connected
            );
            self.as_mut()
                .set_sensor_connected(new_state.sensor_connected);
        }

        let current_error = self.as_ref().error_message().to_string();
        let new_error = new_state.error_message.clone().unwrap_or_default();
        if current_error != new_error {
            if !new_error.is_empty() {
                tracing::warn!("[MonitoringViewModel] Error message: {}", new_error);
            } else {
                tracing::debug!("[MonitoringViewModel] Error cleared");
            }
            self.as_mut().set_error_message(QString::from(&new_error));
        }

        self.update_raw_sensor_data();

        tracing::trace!("[MonitoringViewModel] State update completed");
    }

    /// 清除错误
    pub fn clear_error(mut self: Pin<&mut Self>) {
        tracing::info!("[MonitoringViewModel] User action: clear_error");
        self.as_mut().handle_intent(MonitoringIntent::ClearError);
    }

    /// 重置报警
    pub fn reset_alarm(mut self: Pin<&mut Self>) {
        tracing::info!("[MonitoringViewModel] User action: reset_alarm");
        self.as_mut().handle_intent(MonitoringIntent::ResetAlarm);
    }

    /// 测试方法：更新模拟数据（仅用于开发测试）
    pub fn update_test_data(mut self: Pin<&mut Self>, load: f64, radius: f64, angle: f64) {
        // 创建模拟的传感器数据
        let sensor_data = qt_rust_demo::models::SensorData::new(load, radius, angle);
        
        // 使用简单转换（不依赖配置）
        let processed_data = qt_rust_demo::models::ProcessedData::from_sensor_data(sensor_data, 0);
        
        // 通过 Intent 更新状态
        self.as_mut()
            .handle_intent(MonitoringIntent::ProcessedDataUpdated(processed_data));
    }

    /// 初始化显示管道（通过 Pin 访问内部字段）
    fn init_display_pipeline(self: Pin<&mut Self>, buffer: SharedBuffer) {
        use qt_rust_demo::pipeline::DisplayPipelineConfig;
        use std::time::Duration;

        let config = DisplayPipelineConfig {
            interval: Duration::from_millis(100),
            pipeline_size: 10,
            batch_size: 1,
        };
        let mut pipeline = DisplayPipeline::new(config, buffer);
        pipeline.start(); // 启动显示管道
        *self.display_pipeline.borrow_mut() = Some(Box::new(pipeline));
        tracing::info!("Display pipeline initialized and started in ViewModel");
    }

    /// 手动触发显示更新（供 QML Timer 调用）
    pub fn tick_display(self: Pin<&mut Self>) -> bool {
        tracing::trace!("[MonitoringViewModel] tick_display called");

        let mut pipeline_ref = self.display_pipeline.borrow_mut();
        match pipeline_ref.as_mut() {
            Some(pipeline) => {
                if pipeline.tick() {
                    if let Some(processed_data) = pipeline.get_latest() {
                        tracing::info!("[MonitoringViewModel] 从DisplayPipeline获取数据: load={:.2}吨, radius={:.2}米, angle={:.2}度", 
                            processed_data.current_load, processed_data.working_radius, processed_data.boom_angle);
                        drop(pipeline_ref);
                        // 直接使用 ProcessedData，不再转换为 SensorData
                        self.handle_intent(MonitoringIntent::ProcessedDataUpdated(processed_data));
                        return true;
                    } else {
                        tracing::trace!(
                            "[MonitoringViewModel] tick_display: no new data available"
                        );
                    }
                } else {
                    tracing::trace!(
                        "[MonitoringViewModel] tick_display: pipeline tick returned false"
                    );
                }
                false
            }
            None => {
                tracing::warn!("[MonitoringViewModel] tick_display: pipeline not initialized!");
                false
            }
        }
    }

    /// 从全局管理器获取共享缓冲区并初始化显示管道
    pub fn init_display_pipeline_from_global(self: Pin<&mut Self>) {
        use crate::viewmodel_manager::get_global_shared_buffer;

        tracing::debug!("init_display_pipeline_from_global called");

        match get_global_shared_buffer() {
            Some(buffer) => {
                tracing::info!("Got shared buffer, initializing display pipeline");
                self.init_display_pipeline(buffer);
                tracing::info!("Display pipeline initialized successfully");
            }
            None => {
                tracing::error!("Global shared buffer not available!");
            }
        }
    }

    /// 从全局传感器缓冲区更新原始传感器数据
    pub fn update_raw_sensor_data(mut self: Pin<&mut Self>) {
        use crate::viewmodel_manager::get_global_shared_sensor_buffer;
        let buffer = match get_global_shared_sensor_buffer() {
            Some(b) => b,
            None => return,
        };
        let guard = match buffer.read() {
            Ok(g) => g,
            Err(_) => return,
        };
        let raw_data = guard.get_latest_raw();
        let (ad1, ad2, ad3) = match raw_data {
            Some(data) => data,
            None => return,
        };

        if *self.as_ref().raw_ad1_load() != ad1 {
            self.as_mut().set_raw_ad1_load(ad1);
        }
        if *self.as_ref().raw_ad2_radius() != ad2 {
            self.as_mut().set_raw_ad2_radius(ad2);
        }
        if *self.as_ref().raw_ad3_angle() != ad3 {
            self.as_mut().set_raw_ad3_angle(ad3);
        }
    }
}
