use crate::models::enums::TranslationEngineType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub selected_engine: TranslationEngineType,
    pub factorio_install_path: String,
    pub ui_language: String,
    pub last_mod_path: String,
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub window_width: u32,
    pub window_height: u32,
    #[serde(default)]
    pub api_keys: std::collections::HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();

        assert_eq!(settings.selected_engine, TranslationEngineType::DeepL);
        assert!(settings.factorio_install_path.is_empty());
        assert!(settings.ui_language.is_empty());
        assert!(settings.last_mod_path.is_empty());
        assert!(settings.api_keys.is_empty());
    }

    #[test]
    fn test_app_settings_creation() {
        let mut api_keys = std::collections::HashMap::new();
        api_keys.insert("DeepL".to_string(), "test-key".to_string());

        let settings = AppSettings {
            selected_engine: TranslationEngineType::GoogleTranslate,
            factorio_install_path: "/path/to/factorio".to_string(),
            ui_language: "en".to_string(),
            last_mod_path: "/path/to/mod".to_string(),
            default_source_lang: "en".to_string(),
            default_target_lang: "ja".to_string(),
            window_width: 1024,
            window_height: 768,
            api_keys,
        };

        assert_eq!(
            settings.selected_engine,
            TranslationEngineType::GoogleTranslate
        );
        assert_eq!(settings.factorio_install_path, "/path/to/factorio");
        assert_eq!(settings.window_width, 1024);
        assert_eq!(settings.window_height, 768);
        assert_eq!(
            settings.api_keys.get("DeepL"),
            Some(&"test-key".to_string())
        );
    }

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            selected_engine: TranslationEngineType::DeepL,
            factorio_install_path: String::new(),
            ui_language: "ja".to_string(),
            last_mod_path: String::new(),
            default_source_lang: "en".to_string(),
            default_target_lang: "ja".to_string(),
            window_width: 800,
            window_height: 600,
            api_keys: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("DeepL"));
        assert!(json.contains("800"));

        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.window_width, 800);
    }

    #[test]
    fn test_app_settings_api_keys() {
        let mut settings = AppSettings::default();
        settings
            .api_keys
            .insert("Google".to_string(), "google-key".to_string());

        assert_eq!(settings.api_keys.len(), 1);
        assert_eq!(
            settings.api_keys.get("Google"),
            Some(&"google-key".to_string())
        );
    }
}
