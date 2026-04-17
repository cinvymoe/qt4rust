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
        #[qproperty(f64, min_angle)]
        #[qproperty(f64, max_angle)]
        #[qproperty(i32, main_hook_mode)]
        #[qproperty(i32, aux_hook_mode)]
        type AlarmThresholdViewModel = super::AlarmThresholdViewModelRust;

        #[qinvokable]
        unsafe fn save_thresholds(self: Pin<&mut AlarmThresholdViewModel>) -> bool;

        #[qinvokable]
        unsafe fn reset_to_default(self: Pin<&mut AlarmThresholdViewModel>);
    }
}

use core::pin::Pin;
use sensor_core::HookSwitchMode;

fn mode_to_i32(mode: &HookSwitchMode) -> i32 {
    match mode {
        HookSwitchMode::None => 0,
        HookSwitchMode::NormallyOpen => 1,
        HookSwitchMode::NormallyClosed => 2,
    }
}

fn i32_to_mode(val: i32) -> HookSwitchMode {
    match val {
        1 => HookSwitchMode::NormallyOpen,
        2 => HookSwitchMode::NormallyClosed,
        _ => HookSwitchMode::None,
    }
}

pub struct AlarmThresholdViewModelRust {
    moment_warning_threshold: f64,
    moment_danger_threshold: f64,
    max_load: f64,
    min_angle: f64,
    max_angle: f64,
    main_hook_mode: i32,
    aux_hook_mode: i32,
    calibration_config_path: String,
    alarm_config_path: String,
}

impl Default for AlarmThresholdViewModelRust {
    fn default() -> Self {
        let calibration_config_path = "config/sensor_calibration.toml".to_string();
        let alarm_config_path = "config/alarm_thresholds.toml".to_string();

        // 加载标定配置（获取 max_load）
        let cal_manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(
            &calibration_config_path,
        );
        let calibration = cal_manager.load().unwrap_or_default();

        // 加载报警阈值配置
        let alarm_manager =
            qt_rust_demo::config::alarm_threshold_manager::AlarmThresholdManager::new(
                &alarm_config_path,
            );
        let alarm_thresholds = alarm_manager.load().unwrap_or_default();

        Self {
            moment_warning_threshold: alarm_thresholds.moment.warning_percentage,
            moment_danger_threshold: alarm_thresholds.moment.alarm_percentage,
            max_load: calibration.weight().scale_value,
            min_angle: alarm_thresholds.angle.min_angle,
            max_angle: alarm_thresholds.angle.max_angle,
            main_hook_mode: mode_to_i32(&alarm_thresholds.main_hook_switch.mode),
            aux_hook_mode: mode_to_i32(&alarm_thresholds.aux_hook_switch.mode),
            calibration_config_path,
            alarm_config_path,
        }
    }
}

impl alarm_threshold_bridge::AlarmThresholdViewModel {
    pub fn save_thresholds(self: Pin<&mut Self>) -> bool {
        let mwt = *self.as_ref().moment_warning_threshold();
        let mdt = *self.as_ref().moment_danger_threshold();
        let min_angle = *self.as_ref().min_angle();
        let max_angle = *self.as_ref().max_angle();
        let main_hook_mode_val = *self.as_ref().main_hook_mode();
        let aux_hook_mode_val = *self.as_ref().aux_hook_mode();

        // 保存报警阈值到独立文件
        let alarm_manager =
            qt_rust_demo::config::alarm_threshold_manager::AlarmThresholdManager::new(
                &self.alarm_config_path,
            );
        let mut alarm_thresholds = match alarm_manager.load() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to load alarm thresholds: {:?}", e);
                return false;
            }
        };

        alarm_thresholds.moment.warning_percentage = mwt;
        alarm_thresholds.moment.alarm_percentage = mdt;
        alarm_thresholds.angle.min_angle = min_angle;
        alarm_thresholds.angle.max_angle = max_angle;
        alarm_thresholds.main_hook_switch.mode = i32_to_mode(main_hook_mode_val);
        alarm_thresholds.aux_hook_switch.mode = i32_to_mode(aux_hook_mode_val);

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
        let alarm_thresholds = sensor_core::AlarmThresholds::default();
        let calibration = sensor_core::SensorCalibration::default();

        self.as_mut()
            .set_moment_warning_threshold(alarm_thresholds.moment.warning_percentage);
        self.as_mut()
            .set_moment_danger_threshold(alarm_thresholds.moment.alarm_percentage);
        self.as_mut().set_max_load(calibration.weight().scale_value);
        self.as_mut()
            .set_min_angle(alarm_thresholds.angle.min_angle);
        self.as_mut()
            .set_max_angle(alarm_thresholds.angle.max_angle);
        self.as_mut()
            .set_main_hook_mode(mode_to_i32(&alarm_thresholds.main_hook_switch.mode));
        self.as_mut()
            .set_aux_hook_mode(mode_to_i32(&alarm_thresholds.aux_hook_switch.mode));
    }
}
