pub mod traits;
pub mod locale_manager;
pub mod fluent_backend;

pub use traits::TranslationProvider;
pub use locale_manager::LocaleManager;
pub use fluent_backend::FluentBackend;

use std::path::PathBuf;
use std::sync::Arc;

/// Create translation provider with default configuration
/// Loads the saved locale preference and the fallback locale (zh-CN)
pub fn create_translation_provider(
    config_dir: PathBuf,
    translations_dir: PathBuf,
) -> Result<TranslationProvider, String> {
    let locale_manager = Arc::new(LocaleManager::new(config_dir));
    let backend = Arc::new(FluentBackend::new(locale_manager.clone()));
    
    backend.load_locale("zh-CN", &translations_dir)?;
    
    let saved_locale = locale_manager.get_locale();
    if saved_locale != "zh-CN" {
        backend.load_locale(&saved_locale, &translations_dir)?;
    }
    
    Ok(backend)
}
