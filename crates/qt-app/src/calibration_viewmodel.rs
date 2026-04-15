// 参数校准视图 ViewModel

#[cxx_qt::bridge]
pub mod calibration_viewmodel_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, ad1_load)]
        #[qproperty(f64, ad2_radius)]
        #[qproperty(f64, ad3_angle)]
        #[qproperty(f64, calculated_load)]
        #[qproperty(f64, calculated_radius)]
        #[qproperty(f64, calculated_angle)]
        #[qproperty(bool, sensor_connected)]
        #[qproperty(QString, error_message)]
        type CalibrationViewModel = super::CalibrationViewModelRust;

        /// 清除错误（Intent）
        #[qinvokable]
        unsafe fn clear_error(self: Pin<&mut CalibrationViewModel>);

        /// 从全局传感器缓冲区更新传感器数据（供 QML Timer 调用）
        #[qinvokable]
        unsafe fn update_sensor_data(self: Pin<&mut CalibrationViewModel>);
    }
}

use crate::intents::calibration_intent::CalibrationIntent;
use crate::reducers::calibration_reducer::CalibrationReducer;
use crate::states::calibration_state::CalibrationState;
use core::pin::Pin;
use cxx_qt_lib::QString;

/// CalibrationViewModel 实现
pub struct CalibrationViewModelRust {
    // Qt 属性字段
    ad1_load: f64,
    ad2_radius: f64,
    ad3_angle: f64,
    calculated_load: f64,
    calculated_radius: f64,
    calculated_angle: f64,
    sensor_connected: bool,
    error_message: QString,

    // 内部状态
    reducer: CalibrationReducer,
}

impl Default for CalibrationViewModelRust {
    fn default() -> Self {
        let state = CalibrationState::default();
        Self {
            ad1_load: state.ad1_load,
            ad2_radius: state.ad2_radius,
            ad3_angle: state.ad3_angle,
            calculated_load: state.calculated_load,
            calculated_radius: state.calculated_radius,
            calculated_angle: state.calculated_angle,
            sensor_connected: state.sensor_connected,
            error_message: QString::from(""),
            reducer: CalibrationReducer::new(),
        }
    }
}

impl calibration_viewmodel_bridge::CalibrationViewModel {
    /// 处理意图
    pub fn handle_intent(self: Pin<&mut Self>, intent: CalibrationIntent) {
        tracing::debug!("[CalibrationViewModel] Handling intent: {:?}", intent);

        // 从 Qt 属性重建当前状态
        let current_state = CalibrationState {
            ad1_load: *self.as_ref().ad1_load(),
            ad2_radius: *self.as_ref().ad2_radius(),
            ad3_angle: *self.as_ref().ad3_angle(),
            calculated_load: *self.as_ref().calculated_load(),
            calculated_radius: *self.as_ref().calculated_radius(),
            calculated_angle: *self.as_ref().calculated_angle(),
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
        let new_state = self.reducer.reduce(current_state, intent);

        // 更新状态
        self.update_state(new_state);
    }

    /// 更新状态并触发 Qt 属性变化信号
    fn update_state(mut self: Pin<&mut Self>, new_state: CalibrationState) {
        // 只更新变化的属性
        if *self.as_ref().ad1_load() != new_state.ad1_load {
            tracing::trace!(
                "[CalibrationViewModel] Updating ad1_load: {:.2} -> {:.2}",
                *self.as_ref().ad1_load(),
                new_state.ad1_load
            );
            self.as_mut().set_ad1_load(new_state.ad1_load);
        }
        if *self.as_ref().ad2_radius() != new_state.ad2_radius {
            tracing::trace!(
                "[CalibrationViewModel] Updating ad2_radius: {:.2} -> {:.2}",
                *self.as_ref().ad2_radius(),
                new_state.ad2_radius
            );
            self.as_mut().set_ad2_radius(new_state.ad2_radius);
        }
        if *self.as_ref().ad3_angle() != new_state.ad3_angle {
            tracing::trace!(
                "[CalibrationViewModel] Updating ad3_angle: {:.2} -> {:.2}",
                *self.as_ref().ad3_angle(),
                new_state.ad3_angle
            );
            self.as_mut().set_ad3_angle(new_state.ad3_angle);
        }
        if *self.as_ref().calculated_load() != new_state.calculated_load {
            tracing::trace!(
                "[CalibrationViewModel] Updating calculated_load: {:.2} -> {:.2}",
                *self.as_ref().calculated_load(),
                new_state.calculated_load
            );
            self.as_mut().set_calculated_load(new_state.calculated_load);
        }
        if *self.as_ref().calculated_radius() != new_state.calculated_radius {
            tracing::trace!(
                "[CalibrationViewModel] Updating calculated_radius: {:.2} -> {:.2}",
                *self.as_ref().calculated_radius(),
                new_state.calculated_radius
            );
            self.as_mut()
                .set_calculated_radius(new_state.calculated_radius);
        }
        if *self.as_ref().calculated_angle() != new_state.calculated_angle {
            tracing::trace!(
                "[CalibrationViewModel] Updating calculated_angle: {:.2} -> {:.2}",
                *self.as_ref().calculated_angle(),
                new_state.calculated_angle
            );
            self.as_mut()
                .set_calculated_angle(new_state.calculated_angle);
        }
        if *self.as_ref().sensor_connected() != new_state.sensor_connected {
            tracing::info!(
                "[CalibrationViewModel] Sensor connection changed: {} -> {}",
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
                tracing::warn!("[CalibrationViewModel] Error message: {}", new_error);
            } else {
                tracing::debug!("[CalibrationViewModel] Error cleared");
            }
            self.as_mut().set_error_message(QString::from(&new_error));
        }
    }

    /// 清除错误
    pub fn clear_error(mut self: Pin<&mut Self>) {
        tracing::info!("[CalibrationViewModel] User action: clear_error");
        self.as_mut().handle_intent(CalibrationIntent::ClearError);
    }

    /// 从全局缓冲区更新传感器数据（包含计算后的物理量）
    pub fn update_sensor_data(mut self: Pin<&mut Self>) {
        use crate::viewmodel_manager::{get_global_shared_buffer, get_global_shared_sensor_buffer};

        // 获取处理后数据缓冲区（包含计算后的物理量）
        let processed_buffer = match get_global_shared_buffer() {
            Some(b) => b,
            None => {
                tracing::warn!("[CalibrationViewModel] Global shared buffer not available");
                self.as_mut()
                    .handle_intent(CalibrationIntent::SensorDisconnected);
                return;
            }
        };

        // 获取原始传感器数据缓冲区（包含AD值）
        let sensor_buffer = match get_global_shared_sensor_buffer() {
            Some(b) => b,
            None => {
                tracing::warn!("[CalibrationViewModel] Global sensor buffer not available");
                self.as_mut()
                    .handle_intent(CalibrationIntent::SensorDisconnected);
                return;
            }
        };

        // 读取处理后数据
        let processed_guard = match processed_buffer.read() {
            Ok(g) => g,
            Err(e) => {
                tracing::error!(
                    "[CalibrationViewModel] Failed to read processed buffer: {}",
                    e
                );
                self.as_mut()
                    .handle_intent(CalibrationIntent::SensorDisconnected);
                return;
            }
        };

        let processed_data = match processed_guard.get_latest() {
            Some(data) => data,
            None => {
                tracing::trace!("[CalibrationViewModel] No processed data available yet");
                return;
            }
        };

        drop(processed_guard);

        // 读取原始AD值
        let sensor_guard = match sensor_buffer.read() {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("[CalibrationViewModel] Failed to read sensor buffer: {}", e);
                return;
            }
        };

        let (ad1, ad2, ad3) = match sensor_guard.get_latest_raw() {
            Some(data) => data,
            None => {
                tracing::trace!("[CalibrationViewModel] No sensor data available yet");
                return;
            }
        };

        tracing::trace!(
            "[CalibrationViewModel] Data updated: load={:.2}吨, radius={:.2}米, angle={:.2}度 (AD: {:.2}, {:.2}, {:.2})",
            processed_data.current_load, processed_data.working_radius, processed_data.boom_angle,
            ad1, ad2, ad3
        );

        drop(sensor_guard);

        // 创建包含AD值和计算值的Intent
        self.as_mut().handle_intent(CalibrationIntent::DataUpdated {
            ad1_load: ad1,
            ad2_radius: ad2,
            ad3_angle: ad3,
            calculated_load: processed_data.current_load,
            calculated_radius: processed_data.working_radius,
            calculated_angle: processed_data.boom_angle,
        });
    }
}
