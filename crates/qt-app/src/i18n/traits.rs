use std::sync::Arc;

/// Core trait for translation providers.
/// Implementations must be Send + Sync for thread safety.
pub trait Translate: Send + Sync {
    /// Get translation for key, returns key itself if not found
    fn t(&self, key: &str) -> String;
    
    /// Get translation with variable interpolation
    /// args: slice of (key, value) pairs for Fluent variables
    fn t_with_args(&self, key: &str, args: &[(&str, &str)]) -> String;
    
    /// Get current locale code as owned String
    fn current_locale(&self) -> String;
    
    /// Get list of available locales
    fn available_locales(&self) -> Vec<String>;
    
    /// Switch to different locale, persists preference
    fn set_locale(&self, locale: &str) -> Result<(), String>;
}

/// Thread-safe wrapper around Translate
pub type TranslationProvider = Arc<dyn Translate>;
