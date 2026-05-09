pub mod traits;
pub mod locale_manager;
pub mod fluent_backend;

pub use traits::TranslationProvider;
pub use locale_manager::LocaleManager;
pub use fluent_backend::FluentBackend;

use std::path::PathBuf;
use std::sync::Arc;

/// Create translation provider with default configuration
/// Loads ALL available locales from translations directory
pub fn create_translation_provider(
    config_dir: PathBuf,
    translations_dir: PathBuf,
) -> Result<TranslationProvider, String> {
    let locale_manager = Arc::new(LocaleManager::new(config_dir));
    let backend = Arc::new(FluentBackend::new(locale_manager.clone()));
    
    // Load ALL available locales from translations directory
    // This allows runtime switching without requiring restart
    if translations_dir.exists() {
        for entry in std::fs::read_dir(&translations_dir)
            .map_err(|e| format!("Failed to read translations dir: {}", e))? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "ftl") {
                    // Extract locale name from filename (e.g., "en-US.ftl" -> "en-US")
                    if let Some(locale_name) = path.file_stem().and_then(|s| s.to_str()) {
                        backend.load_locale(locale_name, &translations_dir)?;
                    }
                }
            }
        }
    } else {
        // Fallback: load zh-CN if translations dir doesn't exist
        backend.load_locale("zh-CN", &translations_dir)?;
    }
    
    Ok(backend)
}
