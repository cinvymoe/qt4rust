// settings_viewmodel/calibration/angle_calibration.rs - 角度传感器校准 ViewModel

#[cxx_qt::bridge]
pub mod angle_calibration_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, min_angle)]
        #[qproperty(f64, max_angle)]
        #[qproperty(f64, point1_ad)]
        #[qproperty(f64, point1_angle)]
        #[qproperty(f64, point2_ad)]
        #[qproperty(f64, point2_angle)]
        #[qproperty(f64, current_angle)]
        #[qproperty(f64, current_ad)]
        type AngleCalibrationViewModel = super::AngleCalibrationViewModelRust;

        #[qinvokable]
        unsafe fn save_calibration(self: Pin<&mut AngleCalibrationViewModel>) -> bool;

        #[qinvokable]
        unsafe fn reset_to_default(self: Pin<&mut AngleCalibrationViewModel>);

        #[qinvokable]
        unsafe fn update_current_reading(
            self: Pin<&mut AngleCalibrationViewModel>,
            angle: f64,
            ad: f64,
        );

        #[qinvokable]
        unsafe fn capture_point1(self: Pin<&mut AngleCalibrationViewModel>);

        #[qinvokable]
        unsafe fn capture_point2(self: Pin<&mut AngleCalibrationViewModel>);
    }
}

use core::pin::Pin;

pub struct AngleCalibrationViewModelRust {
    min_angle: f64,
    max_angle: f64,
    point1_ad: f64,
    point1_angle: f64,
    point2_ad: f64,
    point2_angle: f64,
    current_angle: f64,
    current_ad: f64,
    config_path: String,
}

impl Default for AngleCalibrationViewModelRust {
    fn default() -> Self {
        let config_path = "config/sensor_calibration.toml".to_string();
        let manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(&config_path);
        let calibration = manager.load().unwrap_or_default();

        Self {
            min_angle: calibration.angle_zero_value,
            max_angle: calibration.angle_scale_value,
            point1_ad: calibration.angle_zero_ad,
            point1_angle: calibration.angle_zero_value,
            point2_ad: calibration.angle_scale_ad,
            point2_angle: calibration.angle_scale_value,
            current_angle: 0.0,
            current_ad: 0.0,
            config_path,
        }
    }
}

impl angle_calibration_bridge::AngleCalibrationViewModel {
    pub fn save_calibration(self: Pin<&mut Self>) -> bool {
        let p1_ad = *self.as_ref().point1_ad();
        let p1_ang = *self.as_ref().point1_angle();
        let p2_ad = *self.as_ref().point2_ad();
        let p2_ang = *self.as_ref().point2_angle();

        let manager = qt_rust_demo::config::calibration_manager::CalibrationManager::new(&self.config_path);
        let mut calibration = match manager.load() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to load calibration: {:?}", e);
                return false;
            }
        };

        calibration.angle_zero_ad = p1_ad;
        calibration.angle_zero_value = p1_ang;
        calibration.angle_scale_ad = p2_ad;
        calibration.angle_scale_value = p2_ang;

        match manager.save(&calibration) {
            Ok(_) => {
                tracing::info!("Angle calibration saved successfully");
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
        self.as_mut().set_point1_ad(calibration.angle_zero_ad);
        self.as_mut().set_point1_angle(calibration.angle_zero_value);
        self.as_mut().set_point2_ad(calibration.angle_scale_ad);
        self.as_mut()
            .set_point2_angle(calibration.angle_scale_value);
        self.as_mut().set_min_angle(calibration.angle_zero_value);
        self.as_mut().set_max_angle(calibration.angle_scale_value);
    }

    pub fn update_current_reading(mut self: Pin<&mut Self>, angle: f64, ad: f64) {
        self.as_mut().set_current_angle(angle);
        self.as_mut().set_current_ad(ad);
    }

    pub fn capture_point1(mut self: Pin<&mut Self>) {
        let cad = *self.as_ref().current_ad();
        let cangle = *self.as_ref().current_angle();
        self.as_mut().set_point1_ad(cad);
        self.as_mut().set_point1_angle(cangle);
    }

    pub fn capture_point2(mut self: Pin<&mut Self>) {
        let cad = *self.as_ref().current_ad();
        let cangle = *self.as_ref().current_angle();
        self.as_mut().set_point2_ad(cad);
        self.as_mut().set_point2_angle(cangle);
    }
}
