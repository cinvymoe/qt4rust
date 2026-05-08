use crate::i18n::traits::Translate;
use crate::i18n::locale_manager::LocaleManager;
use fluent_bundle::{concurrent::FluentBundle, FluentArgs, FluentResource};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe Fluent bundle type using concurrent memoizer
type ConcurrentBundle = FluentBundle<FluentResource>;

/// Fluent-based translation backend
pub struct FluentBackend {
    bundles: Mutex<HashMap<String, ConcurrentBundle>>,
    current_locale: Mutex<String>,
    locale_manager: Arc<LocaleManager>,
}

impl FluentBackend {
    pub fn new(locale_manager: Arc<LocaleManager>) -> Self {
        Self {
            bundles: Mutex::new(HashMap::new()),
            current_locale: Mutex::new(locale_manager.get_locale()),
            locale_manager,
        }
    }
    
    /// Load translation file for a locale from the given directory
    pub fn load_locale(&self, locale: &str, translations_dir: &Path) -> Result<(), String> {
        let file_path = translations_dir.join(format!("{}.ftl", locale));
        
        let source = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to load {}: {}", locale, e))?;
        
        let resource = FluentResource::try_new(source)
            .map_err(|(_, errors)| {
                let messages: Vec<_> = errors.iter().map(|e| e.to_string()).collect();
                format!("Parse errors in {}: {}", locale, messages.join(", "))
            })?;
        
        let mut bundle: ConcurrentBundle = FluentBundle::new_concurrent(vec![locale.parse().unwrap()]);
        bundle.add_resource(resource)
            .map_err(|e| format!("Bundle error for {}: {:?}", locale, e))?;
        
        self.bundles.lock().unwrap().insert(locale.to_string(), bundle);
        
        Ok(())
    }
}

impl Translate for FluentBackend {
    fn t(&self, key: &str) -> String {
        self.t_with_args(key, &[])
    }
    
    fn t_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let locale = self.current_locale.lock().unwrap().clone();
        let bundles = self.bundles.lock().unwrap();
        
        let Some(bundle) = bundles.get(&locale) else {
            tracing::warn!("Locale bundle not found: {}, returning key", locale);
            return key.to_string();
        };
        
        // Convert dot-separated key to Fluent hyphen format
        // e.g. "monitoring.unit.ton" -> "monitoring-unit-ton"
        let fluent_key = key.replace('.', "-");
        
        let Some(message) = bundle.get_message(&fluent_key) else {
            tracing::warn!("Translation key not found: {}", key);
            return key.to_string();
        };
        
        let Some(pattern) = message.value() else {
            tracing::warn!("Translation key has no value: {}", key);
            return key.to_string();
        };
        
        let mut errors = vec![];
        
        let fluent_args: FluentArgs = args.iter().map(|(k, v)| (*k, *v)).collect();
        let result = if args.is_empty() {
            bundle.format_pattern(pattern, None, &mut errors)
        } else {
            bundle.format_pattern(pattern, Some(&fluent_args), &mut errors)
        };
        
        if !errors.is_empty() {
            tracing::warn!("Translation errors for key {}: {:?}", key, errors);
        }
        
        result.into_owned()
    }
    
    fn current_locale(&self) -> String {
        self.current_locale.lock().unwrap().clone()
    }
    
    fn available_locales(&self) -> Vec<String> {
        self.bundles.lock().unwrap().keys().cloned().collect()
    }
    
    fn set_locale(&self, locale: &str) -> Result<(), String> {
        if !self.bundles.lock().unwrap().contains_key(locale) {
            return Err(format!("Locale not loaded: {}", locale));
        }
        
        *self.current_locale.lock().unwrap() = locale.to_string();
        
        self.locale_manager.set_locale(locale)?;
        
        Ok(())
    }
}
