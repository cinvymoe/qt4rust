use crate::i18n::traits::TranslationProvider;
use std::sync::OnceLock;

static GLOBAL_TRANSLATION_PROVIDER: OnceLock<TranslationProvider> = OnceLock::new();

pub fn set_global_translation_provider(provider: TranslationProvider) {
    let _ = GLOBAL_TRANSLATION_PROVIDER.set(provider);
}

pub fn get_translation_provider() -> Option<&'static TranslationProvider> {
    GLOBAL_TRANSLATION_PROVIDER.get()
}

#[cxx_qt::bridge]
pub mod translation_bridge {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qvector.h");
        type QString = cxx_qt_lib::QString;
        type QVector_QString = cxx_qt_lib::QVector<QString>;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, current_locale)]
        type TranslationBridge = super::TranslationBridgeRust;

        #[qinvokable]
        fn translate(self: &TranslationBridge, key: &QString) -> QString;

        #[qinvokable]
        fn translate_with_args(self: &TranslationBridge, key: &QString, args_json: &QString) -> QString;

        #[qinvokable]
        fn available_locales(self: &TranslationBridge) -> QVector_QString;

        #[qinvokable]
        fn set_locale(self: Pin<&mut TranslationBridge>, locale: &QString) -> bool;
    }
}

use core::pin::Pin;
use cxx_qt_lib::{QString, QVector};
use crate::translation_bridge::translation_bridge::QVector_QString;

pub struct TranslationBridgeRust {
    current_locale: QString,
}

impl Default for TranslationBridgeRust {
    fn default() -> Self {
        let locale = get_translation_provider()
            .map(|p| p.current_locale())
            .unwrap_or_else(|| "zh-CN".to_string());
        Self {
            current_locale: QString::from(&locale),
        }
    }
}

impl translation_bridge::TranslationBridge {
    pub fn translate(self: &translation_bridge::TranslationBridge, key: &QString) -> QString {
        let key_str = key.to_string();
        let result = get_translation_provider()
            .map(|p| p.t(&key_str))
            .unwrap_or_else(|| key_str);
        QString::from(&result)
    }

    pub fn translate_with_args(
        self: &translation_bridge::TranslationBridge,
        key: &QString,
        args_json: &QString,
    ) -> QString {
        let key_str = key.to_string();
        let args_str = args_json.to_string();

        let parsed: serde_json::Result<std::collections::HashMap<String, String>> =
            serde_json::from_str(&args_str);

        let result = match parsed {
            Ok(map) => {
                let args_vec: Vec<(&str, &str)> = map.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                get_translation_provider()
                    .map(|p| p.t_with_args(&key_str, &args_vec))
                    .unwrap_or_else(|| key_str.clone())
            }
            Err(e) => {
                tracing::warn!("Failed to parse translation args JSON: {}", e);
                get_translation_provider()
                    .map(|p| p.t(&key_str))
                    .unwrap_or_else(|| key_str)
            }
        };

        QString::from(&result)
    }

    pub fn available_locales(self: &translation_bridge::TranslationBridge) -> QVector_QString {
        let locales: Vec<QString> = get_translation_provider()
            .map(|p| p.available_locales().into_iter().map(|s| QString::from(&s)).collect())
            .unwrap_or_default();
        QVector::from_iter(locales.iter())
    }

    pub fn set_locale(mut self: Pin<&mut translation_bridge::TranslationBridge>, locale: &QString) -> bool {
        let locale_str = locale.to_string();
        
        match get_translation_provider() {
            Some(provider) => {
                match provider.set_locale(&locale_str) {
                    Ok(()) => {
                        self.as_mut().set_current_locale(QString::from(&locale_str));
                        tracing::info!("Locale switched to: {}", locale_str);
                        true
                    }
                    Err(e) => {
                        tracing::error!("Failed to set locale: {}", e);
                        false
                    }
                }
            }
            None => {
                tracing::error!("Translation provider not initialized");
                false
            }
        }
    }
}
