// settings_viewmodel/settings_main.rs - 主设置 ViewModel

#[cxx_qt::bridge]
pub mod settings_main_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, current_page)]
        #[qproperty(bool, has_unsaved_changes)]
        type SettingsViewModel = super::SettingsViewModelRust;

        #[qinvokable]
        unsafe fn navigate_to(self: Pin<&mut SettingsViewModel>, page: QString);

        #[qinvokable]
        unsafe fn save_all(self: Pin<&mut SettingsViewModel>) -> bool;

        #[qinvokable]
        unsafe fn discard_changes(self: Pin<&mut SettingsViewModel>);

        #[qinvokable]
        unsafe fn export_config(self: Pin<&mut SettingsViewModel>, path: QString) -> bool;

        #[qinvokable]
        unsafe fn import_config(self: Pin<&mut SettingsViewModel>, path: QString) -> bool;
    }
}

use core::pin::Pin;
use cxx_qt_lib::QString;

pub struct SettingsViewModelRust {
    current_page: QString,
    has_unsaved_changes: bool,
}

impl Default for SettingsViewModelRust {
    fn default() -> Self {
        Self {
            current_page: QString::from("overview"),
            has_unsaved_changes: false,
        }
    }
}

impl settings_main_bridge::SettingsViewModel {
    pub fn navigate_to(mut self: Pin<&mut Self>, page: QString) {
        self.as_mut().set_current_page(page);
    }

    pub fn save_all(mut self: Pin<&mut Self>) -> bool {
        // 这里可以触发所有子 ViewModel 的保存操作
        tracing::info!("Saving all settings");
        self.as_mut().set_has_unsaved_changes(false);
        true
    }

    pub fn discard_changes(mut self: Pin<&mut Self>) {
        tracing::info!("Discarding all changes");
        self.as_mut().set_has_unsaved_changes(false);
    }

    pub fn export_config(self: Pin<&mut Self>, path: QString) -> bool {
        let path_str = path.to_string();
        tracing::info!("Exporting configuration to: {}", path_str);

        // 实现配置导出逻辑
        match std::fs::copy("config/sensor_calibration.toml", &path_str) {
            Ok(_) => {
                tracing::info!("Configuration exported successfully");
                true
            }
            Err(e) => {
                tracing::error!("Failed to export configuration: {:?}", e);
                false
            }
        }
    }

    pub fn import_config(mut self: Pin<&mut Self>, path: QString) -> bool {
        let path_str = path.to_string();
        tracing::info!("Importing configuration from: {}", path_str);

        // 实现配置导入逻辑
        match std::fs::copy(&path_str, "config/sensor_calibration.toml") {
            Ok(_) => {
                tracing::info!("Configuration imported successfully");
                self.as_mut().set_has_unsaved_changes(true);
                true
            }
            Err(e) => {
                tracing::error!("Failed to import configuration: {:?}", e);
                false
            }
        }
    }
}
