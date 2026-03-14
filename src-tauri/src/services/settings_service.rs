use std::fs;
use std::path::{Path, PathBuf};
use serde_json;
use crate::models::settings::AppSettings;
use crate::models::enums::TranslationEngineType;
use keyring::Entry;

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
            if let Ok(content) = fs::read_to_string(&self.settings_path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    return settings;
                }
            }
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

        let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
        fs::write(&self.settings_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save_api_key(&self, engine: &str, key: &str) -> Result<(), String> {
        let entry = Entry::new("factoriomodtranslator", engine).map_err(|e| e.to_string())?;
        entry.set_password(key).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_api_key(&self, engine: &str) -> Option<String> {
        let entry = Entry::new("factoriomodtranslator", engine).ok()?;
        entry.get_password().ok()
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
}
