// settings_viewmodel/calibration/load_calibration.rs - 载荷传感器校准 ViewModel

#[cxx_qt::bridge]
pub mod load_calibration_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, calibration_multiplier)]
        #[qproperty(f64, point1_ad)]
        #[qproperty(f64, point1_weight)]
        #[qproperty(f64, point2_ad)]
        #[qproperty(f64, point2_weight)]
        #[qproperty(f64, current_load)]
        #[qproperty(f64, current_ad)]
        type LoadCalibrationViewModel = super::LoadCalibrationViewModelRust;

        #[qinvokable]
        unsafe fn save_calibration(self: Pin<&mut LoadCalibrationViewModel>) -> bool;

        #[qinvokable]
        unsafe fn reset_to_default(self: Pin<&mut LoadCalibrationViewModel>);

        #[qinvokable]
        unsafe fn update_current_reading(
            self: Pin<&mut LoadCalibrationViewModel>,
            load: f64,
            ad: f64,
        );

        #[qinvokable]
        unsafe fn capture_point1(self: Pin<&mut LoadCalibrationViewModel>);

        #[qinvokable]
        unsafe fn capture_point2(self: Pin<&mut LoadCalibrationViewModel>);
    }
}

use core::pin::Pin;

pub struct LoadCalibrationViewModelRust {
    calibration_multiplier: i32,
    point1_ad: f64,
    point1_weight: f64,
    point2_ad: f64,
    point2_weight: f64,
    current_load: f64,
    current_ad: f64,
    config_path: String,
}

impl Default for LoadCalibrationViewModelRust {
    fn default() -> Self {
        let config_path = "config/sensor_calibration.toml".to_string();
        let manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(&config_path);
        let calibration = manager.load().unwrap_or_default();

        Self {
            calibration_multiplier: 1,
            point1_ad: calibration.weight_zero_ad,
            point1_weight: calibration.weight_zero_value,
            point2_ad: calibration.weight_scale_ad,
            point2_weight: calibration.weight_scale_value,
            current_load: 0.0,
            current_ad: 0.0,
            config_path,
        }
    }
}

impl load_calibration_bridge::LoadCalibrationViewModel {
    pub fn save_calibration(self: Pin<&mut Self>) -> bool {
        let p1_ad = *self.as_ref().point1_ad();
        let p1_wt = *self.as_ref().point1_weight();
        let p2_ad = *self.as_ref().point2_ad();
        let p2_wt = *self.as_ref().point2_weight();

        let manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(&self.config_path);
        let mut calibration = match manager.load() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to load calibration: {:?}", e);
                return false;
            }
        };

        calibration.weight_zero_ad = p1_ad;
        calibration.weight_zero_value = p1_wt;
        calibration.weight_scale_ad = p2_ad;
        calibration.weight_scale_value = p2_wt;

        match manager.save(&calibration) {
            Ok(_) => {
                tracing::info!("Load calibration saved successfully");
                true
            }
            Err(e) => {
                tracing::error!("Failed to save calibration: {:?}", e);
                false
            }
        }
    }

    pub fn reset_to_default(mut self: Pin<&mut Self>) {
        let calibration = qt_rust_demo::models::sensor_calibration::SensorCalibration::default();
        self.as_mut().set_point1_ad(calibration.weight_zero_ad);
        self.as_mut()
            .set_point1_weight(calibration.weight_zero_value);
        self.as_mut().set_point2_ad(calibration.weight_scale_ad);
        self.as_mut()
            .set_point2_weight(calibration.weight_scale_value);
        self.as_mut().set_calibration_multiplier(1);
    }

    pub fn update_current_reading(mut self: Pin<&mut Self>, load: f64, ad: f64) {
        self.as_mut().set_current_load(load);
        self.as_mut().set_current_ad(ad);
    }

    pub fn capture_point1(mut self: Pin<&mut Self>) {
        let cad = *self.as_ref().current_ad();
        let cload = *self.as_ref().current_load();
        self.as_mut().set_point1_ad(cad);
        self.as_mut().set_point1_weight(cload);
    }

    pub fn capture_point2(mut self: Pin<&mut Self>) {
        let cad = *self.as_ref().current_ad();
        let cload = *self.as_ref().current_load();
        self.as_mut().set_point2_ad(cad);
        self.as_mut().set_point2_weight(cload);
    }
}
