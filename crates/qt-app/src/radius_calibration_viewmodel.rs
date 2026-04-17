// radius_calibration_viewmodel.rs - 半径传感器校准 ViewModel
// 由于 cxx-qt 限制，需要在 src/ 根目录定义

#[cxx_qt::bridge]
pub mod radius_calibration_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, min_radius)]
        #[qproperty(f64, max_radius)]
        #[qproperty(f64, point1_ad)]
        #[qproperty(f64, point1_radius)]
        #[qproperty(f64, point2_ad)]
        #[qproperty(f64, point2_radius)]
        #[qproperty(f64, current_radius)]
        #[qproperty(f64, current_ad)]
        type RadiusCalibrationViewModel = super::RadiusCalibrationViewModelRust;

        #[qinvokable]
        unsafe fn save_calibration(self: Pin<&mut RadiusCalibrationViewModel>) -> bool;

        #[qinvokable]
        unsafe fn reset_to_default(self: Pin<&mut RadiusCalibrationViewModel>);

        #[qinvokable]
        unsafe fn update_current_reading(
            self: Pin<&mut RadiusCalibrationViewModel>,
            radius: f64,
            ad: f64,
        );

        #[qinvokable]
        unsafe fn capture_point1(self: Pin<&mut RadiusCalibrationViewModel>);

        #[qinvokable]
        unsafe fn capture_point2(self: Pin<&mut RadiusCalibrationViewModel>);
    }
}

use core::pin::Pin;

pub struct RadiusCalibrationViewModelRust {
    min_radius: f64,
    max_radius: f64,
    point1_ad: f64,
    point1_radius: f64,
    point2_ad: f64,
    point2_radius: f64,
    current_radius: f64,
    current_ad: f64,
    config_path: String,
}

impl Default for RadiusCalibrationViewModelRust {
    fn default() -> Self {
        let config_path = "config/sensor_calibration.toml".to_string();
        let manager =
            qt_rust_demo::config::calibration_manager::CalibrationManager::new(&config_path);
        let calibration = manager.load().unwrap_or_default();

        Self {
            min_radius: calibration.radius().zero_value,
            max_radius: calibration.radius().scale_value,
            point1_ad: calibration.radius().zero_ad,
            point1_radius: calibration.radius().zero_value,
            point2_ad: calibration.radius().scale_ad,
            point2_radius: calibration.radius().scale_value,
            current_radius: 0.0,
            current_ad: 0.0,
            config_path,
        }
    }
}

impl radius_calibration_bridge::RadiusCalibrationViewModel {
    pub fn save_calibration(self: Pin<&mut Self>) -> bool {
        let p1_ad = *self.as_ref().point1_ad();
        let p1_rad = *self.as_ref().point1_radius();
        let p2_ad = *self.as_ref().point2_ad();
        let p2_rad = *self.as_ref().point2_radius();

        let manager =
            qt_rust_demo::config::calibration_manager::CalibrationManager::new(&self.config_path);
        let mut calibration = match manager.load() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to load calibration: {:?}", e);
                return false;
            }
        };

        calibration.set_calibration(
            "radius",
            sensor_core::SensorCalibrationParams {
                zero_ad: p1_ad,
                zero_value: p1_rad,
                scale_ad: p2_ad,
                scale_value: p2_rad,
                multiplier: 1.0,
                actual_multiplier: 1.0,
            },
        );

        match manager.save(&calibration) {
            Ok(_) => {
                tracing::info!("Radius calibration saved successfully");
                true
            }
            Err(e) => {
                tracing::error!("Failed to save calibration: {:?}", e);
                false
            }
        }
    }

    pub fn reset_to_default(mut self: Pin<&mut Self>) {
        let calibration = sensor_core::SensorCalibration::default();
        self.as_mut().set_point1_ad(calibration.radius().zero_ad);
        self.as_mut()
            .set_point1_radius(calibration.radius().zero_value);
        self.as_mut().set_point2_ad(calibration.radius().scale_ad);
        self.as_mut()
            .set_point2_radius(calibration.radius().scale_value);
        self.as_mut()
            .set_min_radius(calibration.radius().zero_value);
        self.as_mut()
            .set_max_radius(calibration.radius().scale_value);
    }

    pub fn update_current_reading(mut self: Pin<&mut Self>, radius: f64, ad: f64) {
        self.as_mut().set_current_radius(radius);
        self.as_mut().set_current_ad(ad);
    }

    pub fn capture_point1(mut self: Pin<&mut Self>) {
        let cad = *self.as_ref().current_ad();
        let cradius = *self.as_ref().current_radius();
        self.as_mut().set_point1_ad(cad);
        self.as_mut().set_point1_radius(cradius);
    }

    pub fn capture_point2(mut self: Pin<&mut Self>) {
        let cad = *self.as_ref().current_ad();
        let cradius = *self.as_ref().current_radius();
        self.as_mut().set_point2_ad(cad);
        self.as_mut().set_point2_radius(cradius);
    }
}
