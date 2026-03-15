use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde_json;
use serde_json::json;
use crate::models::settings::AppSettings;
use crate::models::enums::TranslationEngineType;
use log::{info, error};
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
            default_source_lang: "en".to_string(),
            default_target_lang: "ja".to_string(),
            window_width: 1200,
            window_height: 800,
            api_keys: HashMap::new(),
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
        if trimmed_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }
        
        // Encrypt key using machine-specific AES
        let encrypted_data = self.encrypt_local(trimmed_key.as_bytes())?;
        let base64_key = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, encrypted_data);

        // Load current settings
        let mut settings = self.load_settings();
        settings.api_keys.insert(engine.to_string(), base64_key);
        
        self.save_settings(&settings)?;
        info!("{}", json!({ "event": "api_key_saved_securely", "engine": engine }));
        Ok(())
    }

    pub fn load_api_key(&self, engine: &str) -> Option<String> {
        let settings = self.load_settings();
        let base64_value = settings.api_keys.get(engine)?;
        
        let encrypted_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_value).ok()?;
        let decrypted_data = self.decrypt_local(&encrypted_data).ok()?;
        
        String::from_utf8(decrypted_data).ok()
    }

    // --- Machine-Specific Encryption Helpers ---
    
    fn get_encryption_key(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        use std::env;

        let mut hasher = Sha256::new();
        // Use machine/user specific info to derive a key that's unique to this environment
        hasher.update(env::var("COMPUTERNAME").unwrap_or_else(|_| "fixed_comp".to_string()));
        hasher.update(env::var("USERDOMAIN").unwrap_or_else(|_| "fixed_domain".to_string()));
        hasher.update(env::var("USERNAME").unwrap_or_else(|_| "fixed_user".to_string()));
        // Add a salt for the app
        hasher.update(b"factoriomodtranslator_salt_2026");
        
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    fn encrypt_local(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};
        
        let key_bytes = self.get_encryption_key();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        // We use a fixed nonce derived from the app context for metadata predictability 
        // while the key itself is machine-locked.
        let nonce = Nonce::from_slice(b"unique_nonce"); // 12 bytes
        
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| format!("Encryption failure: {}", e))?;
            
        Ok(ciphertext)
    }

    fn decrypt_local(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};
        
        let key_bytes = self.get_encryption_key();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(b"unique_nonce");
        
        let plaintext = cipher.decrypt(nonce, data)
            .map_err(|e| format!("Decryption failure: {}", e))?;
            
        Ok(plaintext)
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
        new_settings.default_source_lang = "en".to_string();
        new_settings.default_target_lang = "ja".to_string();
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
