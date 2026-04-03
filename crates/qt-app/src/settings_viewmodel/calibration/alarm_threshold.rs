// settings_viewmodel/calibration/alarm_threshold.rs - 报警阈值设置 ViewModel

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
        #[qproperty(f64, max_angle)]
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
    max_angle: f64,
    config_path: String,
}

impl Default for AlarmThresholdViewModelRust {
    fn default() -> Self {
        let config_path = "config/sensor_calibration.toml".to_string();
        let manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(&config_path);
        let calibration = manager.load().unwrap_or_default();

        Self {
            moment_warning_threshold: calibration.moment_warning_percentage,
            moment_danger_threshold: calibration.moment_alarm_percentage,
            max_load: calibration.weight_scale_value,
            max_angle: calibration.angle_alarm_value,
            config_path,
        }
    }
}

impl alarm_threshold_bridge::AlarmThresholdViewModel {
    pub fn save_thresholds(self: Pin<&mut Self>) -> bool {
        let mwt = *self.as_ref().moment_warning_threshold();
        let mdt = *self.as_ref().moment_danger_threshold();
        let ma = *self.as_ref().max_angle();

        let manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(&self.config_path);
        let mut calibration = match manager.load() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to load calibration: {:?}", e);
                return false;
            }
        };

        calibration.moment_warning_percentage = mwt;
        calibration.moment_alarm_percentage = mdt;
        calibration.angle_warning_value = ma;
        calibration.angle_alarm_value = ma;

        match manager.save(&calibration) {
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
        let calibration = qt_rust_demo::models::sensor_calibration::SensorCalibration::default();
        self.as_mut()
            .set_moment_warning_threshold(calibration.moment_warning_percentage);
        self.as_mut()
            .set_moment_danger_threshold(calibration.moment_alarm_percentage);
        self.as_mut().set_max_load(calibration.weight_scale_value);
        self.as_mut().set_max_angle(calibration.angle_alarm_value);
    }
}
