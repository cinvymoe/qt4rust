// alarm_threshold_viewmodel.rs - 报警阈值设置 ViewModel
// 由于 cxx-qt 限制，需要在 src/ 根目录定义

#[cxx_qt::bridge]
pub mod alarm_threshold_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, moment_warning_threshold)]
        #[qproperty(f64, moment_danger_threshold)]
        #[qproperty(f64, max_load)]
        type AlarmThresholdViewModel = super::AlarmThresholdViewModelRust;

        #[qinvokable]
        unsafe fn save_thresholds(self: Pin<&mut AlarmThresholdViewModel>) -> bool;

        #[qinvokable]
        unsafe fn reset_to_default(self: Pin<&mut AlarmThresholdViewModel>);
    }
}

use core::pin::Pin;

pub struct AlarmThresholdViewModelRust {
    moment_warning_threshold: f64,
    moment_danger_threshold: f64,
    max_load: f64,
    calibration_config_path: String,
    alarm_config_path: String,
}

impl Default for AlarmThresholdViewModelRust {
    fn default() -> Self {
        let calibration_config_path = "config/sensor_calibration.toml".to_string();
        let alarm_config_path = "config/alarm_thresholds.toml".to_string();
        
        // 加载标定配置（获取 max_load）
        let cal_manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(&calibration_config_path);
        let calibration = cal_manager.load().unwrap_or_default();
        
        // 加载报警阈值配置
        let alarm_manager = qt_rust_demo::config::alarm_threshold_manager::AlarmThresholdManager::new(&alarm_config_path);
        let alarm_thresholds = alarm_manager.load().unwrap_or_default();

        Self {
            moment_warning_threshold: alarm_thresholds.moment.warning_percentage,
            moment_danger_threshold: alarm_thresholds.moment.alarm_percentage,
            max_load: calibration.weight.scale_value,
            calibration_config_path,
            alarm_config_path,
        }
    }
}

impl alarm_threshold_bridge::AlarmThresholdViewModel {
    pub fn save_thresholds(self: Pin<&mut Self>) -> bool {
        let mwt = *self.as_ref().moment_warning_threshold();
        let mdt = *self.as_ref().moment_danger_threshold();

        // 保存报警阈值到独立文件
        let alarm_manager = qt_rust_demo::config::alarm_threshold_manager::AlarmThresholdManager::new(&self.alarm_config_path);
        let mut alarm_thresholds = match alarm_manager.load() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to load alarm thresholds: {:?}", e);
                return false;
            }
        };

        alarm_thresholds.moment.warning_percentage = mwt;
        alarm_thresholds.moment.alarm_percentage = mdt;

        match alarm_manager.save(&alarm_thresholds) {
            Ok(_) => {
                tracing::info!("Alarm thresholds saved successfully");
                true
            }
            Err(e) => {
                tracing::error!("Failed to save thresholds: {:?}", e);
                false
            }
        }
    }

    pub fn reset_to_default(mut self: Pin<&mut Self>) {
        let alarm_thresholds = qt_rust_demo::models::sensor_calibration::AlarmThresholds::default();
        let calibration = qt_rust_demo::models::sensor_calibration::SensorCalibration::default();
        
        self.as_mut()
            .set_moment_warning_threshold(alarm_thresholds.moment.warning_percentage);
        self.as_mut()
            .set_moment_danger_threshold(alarm_thresholds.moment.alarm_percentage);
        self.as_mut().set_max_load(calibration.weight.scale_value);
    }
}
