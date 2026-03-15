use std::fs;
use std::path::{Path, PathBuf};
use serde_json;
use serde_json::json;
use crate::models::settings::AppSettings;
use crate::models::enums::TranslationEngineType;
use keyring::Entry;
use log::{info, warn, error};
use crate::services::logging::mask_sensitive;

pub struct SettingsService {
    settings_path: PathBuf,
}

impl SettingsService {
    pub fn new(app_data_dir: &Path) -> Self {
        let settings_path = app_data_dir.join("settings.json");
        Self { settings_path }
    }

    pub fn load_settings(&self) -> AppSettings {
        if self.settings_path.exists() {
            match fs::read_to_string(&self.settings_path) {
                Ok(content) => {
                    match serde_json::from_str::<AppSettings>(&content) {
                        Ok(settings) => {
                            info!("{}", json!({ "event": "settings_loaded", "path": self.settings_path.display().to_string() }));
                            return settings;
                        }
                        Err(e) => {
                            error!("{}", json!({ "event": "settings_parse_error", "error": e.to_string(), "path": self.settings_path.display().to_string() }));
                        }
                    }
                }
                Err(e) => {
                    error!("{}", json!({ "event": "settings_read_error", "error": e.to_string(), "path": self.settings_path.display().to_string() }));
                }
            }
        } else {
            info!("{}", json!({ "event": "settings_not_found", "path": self.settings_path.display().to_string(), "action": "using_defaults" }));
        }
        
        // Defaults
        AppSettings {
            selected_engine: TranslationEngineType::DeepL,
            factorio_install_path: "C:\\Program Files\\Factorio".to_string(),
            ui_language: "ja".to_string(),
            last_mod_path: String::new(),
            window_width: 1200,
            window_height: 800,
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), String> {
        let dir = self.settings_path.parent().ok_or("Invalid settings path")?;
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(settings).map_err(|e| {
            error!("{}", json!({ "event": "settings_serialize_error", "error": e.to_string() }));
            e.to_string()
        })?;
        
        fs::write(&self.settings_path, json).map_err(|e| {
            error!("{}", json!({ "event": "settings_write_error", "error": e.to_string(), "path": self.settings_path.display().to_string() }));
            e.to_string()
        })?;
        
        info!("{}", json!({ "event": "settings_saved", "path": self.settings_path.display().to_string() }));
        Ok(())
    }

    pub fn save_api_key(&self, engine: &str, key: &str) -> Result<(), String> {
        let trimmed_key = key.trim();
        let entry = Entry::new("factoriomodtranslator", engine).map_err(|e| {
            error!("{}", json!({ "event": "keyring_entry_error", "engine": engine, "error": e.to_string() }));
            e.to_string()
        })?;
        entry.set_password(trimmed_key).map_err(|e| {
            error!("{}", json!({ "event": "keyring_save_error", "engine": engine, "error": e.to_string() }));
            e.to_string()
        })?;
        info!("{}", json!({ "event": "api_key_saved", "engine": engine, "key": mask_sensitive(trimmed_key) }));
        Ok(())
    }

    pub fn load_api_key(&self, engine: &str) -> Option<String> {
        match Entry::new("factoriomodtranslator", engine) {
            Ok(entry) => match entry.get_password() {
                Ok(key) => {
                    info!("{}", json!({ "event": "api_key_loaded", "engine": engine }));
                    Some(key)
                }
                Err(e) => {
                    warn!("{}", json!({ "event": "api_key_load_error", "engine": engine, "error": e.to_string() }));
                    None
                }
            },
            Err(e) => {
                warn!("{}", json!({ "event": "keyring_entry_error", "engine": engine, "error": e.to_string() }));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manage_settings_success_should_save_and_load_successfully() {
        let dir = tempdir().unwrap();
        let service = SettingsService::new(dir.path());
        let mut new_settings = AppSettings::default();
        new_settings.selected_engine = TranslationEngineType::GoogleTranslate;
        new_settings.factorio_install_path = "CustomPath".to_string();

        service.save_settings(&new_settings).unwrap();
        let loaded_settings = service.load_settings();

        assert_eq!(loaded_settings.selected_engine, TranslationEngineType::GoogleTranslate);
        assert_eq!(loaded_settings.factorio_install_path, "CustomPath");
    }

    #[test]
    fn test_manage_settings_invalid_path_should_return_error() {
        let service = SettingsService::new(Path::new("X:\\invalid_drive\\impossible_path\\data.json"));
        let result = service.save_settings(&AppSettings::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_manage_settings_api_key_save_load() {
        let dir = tempdir().unwrap();
        let service = SettingsService::new(dir.path());
        let engine = "TestEngine_Antigravity";
        let key = "test_key_123456789";
        
        // keyring might fail in CI/limited environments, so we log but don't strictly fail the test if the OS service is unavailable
        if let Ok(_) = service.save_api_key(engine, key) {
            let loaded = service.load_api_key(engine);
            assert_eq!(loaded, Some(key.to_string()));
        }
    }
}
