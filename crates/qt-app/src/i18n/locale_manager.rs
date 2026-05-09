use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

const DEFAULT_LOCALE: &str = "zh-CN";
const CONFIG_FILENAME: &str = "app_config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings {
                language: DEFAULT_LOCALE.to_string(),
            },
        }
    }
}

/// Manages language preference and persistence to config file
pub struct LocaleManager {
    config: RwLock<AppConfig>,
    config_path: PathBuf,
}

impl LocaleManager {
    pub fn new(config_dir: PathBuf) -> Self {
        let config_path = config_dir.join(CONFIG_FILENAME);
        let config = Self::load_config(&config_path).unwrap_or_default();
        
        Self {
            config: RwLock::new(config),
            config_path,
        }
    }
    
    fn load_config(path: &PathBuf) -> Option<AppConfig> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }
    
    pub fn get_locale(&self) -> String {
        self.config.read().unwrap().app.language.clone()
    }
    
    pub fn set_locale(&self, locale: &str) -> Result<(), String> {
        let mut config = self.config.write().unwrap();
        config.app.language = locale.to_string();
        
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        let content = toml::to_string_pretty(&*config)
            .map_err(|e| e.to_string())?;
        std::fs::write(&self.config_path, content)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
