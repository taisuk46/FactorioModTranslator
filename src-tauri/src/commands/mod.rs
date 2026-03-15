use tauri::{AppHandle, Manager, State, Emitter};
use std::path::PathBuf;

use crate::models::mod_info::ModInfo;
use crate::models::enums::{TranslationMode, TranslationEngineType};
use crate::models::translation::{TranslationItem, TranslationRecord};
use crate::models::settings::AppSettings;
use crate::models::glossary::GlossaryEntry;

use crate::services::mod_loader::ModLoader;
use crate::services::settings_service::SettingsService;
use crate::services::glossary_service::GlossaryService;
use crate::services::translation_history_service::TranslationHistoryService;
use crate::services::vanilla_translation_service::VanillaTranslationService;
use crate::services::orchestrator::TranslationOrchestrator;
use crate::services::translation_engines::{DeepLTranslationEngine, GoogleTranslationEngine, TranslationEngine};
use crate::services::localization_service::LocalizationService;
use crate::services::logging::{LogContext, mask_sensitive};
use log::info;
use serde_json::json;

pub struct AppState {
    pub vanilla_service: tokio::sync::Mutex<VanillaTranslationService>,
    pub glossary_service: tokio::sync::Mutex<GlossaryService>,
    pub history_service: TranslationHistoryService,
    pub localization_service: LocalizationService,
}

#[tauri::command]
pub async fn get_localized_strings(state: State<'_, AppState>, lang: String) -> Result<std::collections::HashMap<String, String>, String> {
    Ok(state.localization_service.get_all_translations(&lang))
}

#[tauri::command]
pub async fn load_mod(path: String) -> Result<ModInfo, String> {
    let ctx = LogContext::new("load_mod");
    info!("{}", json!({ "request_id": ctx.request_id, "path": path }));
    
    let res = ModLoader::load_from_folder(&path)
        .or_else(|_| ModLoader::load_from_zip(&path));
    
    match &res {
        Ok(info) => {
            info!("{}", json!({ "request_id": ctx.request_id, "title": info.title, "version": info.version }));
            ctx.complete();
        },
        Err(e) => ctx.error(e),
    }
    res
}

#[tauri::command]
pub async fn translate_mod(
    app: AppHandle,
    state: State<'_, AppState>,
    mod_info: ModInfo,
    mode: TranslationMode,
    source_lang: String,
    target_lang: String,
    engine_type: TranslationEngineType,
) -> Result<Vec<TranslationItem>, String> {
    let ctx = LogContext::new("translate_mod");
    info!("{}", json!({
        "request_id": ctx.request_id,
        "mod": mod_info.name,
        "engine": format!("{:?}", engine_type),
        "source": source_lang,
        "target": target_lang
    }));

    let app_data = app.path().app_local_data_dir().unwrap_or(PathBuf::from("."));
    let settings = SettingsService::new(&app_data);
    
    let engine: Box<dyn TranslationEngine> = match engine_type {
        TranslationEngineType::DeepL => {
            let key = settings.load_api_key("DeepL").ok_or("DeepL API key not found")?;
            Box::new(DeepLTranslationEngine::new(key))
        }
        TranslationEngineType::GoogleTranslate => {
            let key = settings.load_api_key("Google").ok_or("Google API key not found")?;
            Box::new(GoogleTranslationEngine::new(key))
        }
    };

    let glossary = state.glossary_service.lock().await;
    let vanilla = state.vanilla_service.lock().await;
    
    let orchestrator = TranslationOrchestrator::new(
        engine.as_ref(),
        &vanilla,
        &glossary,
        &state.history_service,
    );

    let res = orchestrator.execute_translation(
        &mod_info,
        mode,
        &source_lang,
        &target_lang,
        |p| { let _ = app.emit("translation-progress", p); },
    ).await;

    match &res {
        Ok(items) => {
            info!("{}", json!({ "request_id": ctx.request_id, "count": items.len() }));
            ctx.complete();
        },
        Err(e) => ctx.error(e),
    }
    res
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> AppSettings {
    info!("Command: get_settings");
    let app_data = app.path().app_local_data_dir().unwrap_or(PathBuf::from("."));
    SettingsService::new(&app_data).load_settings()
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let app_data = app.path().app_local_data_dir().unwrap_or(PathBuf::from("."));
    SettingsService::new(&app_data).save_settings(&settings)
}

#[tauri::command]
pub async fn save_api_key(app: AppHandle, engine: String, key: String) -> Result<(), String> {
    info!("{}", json!({ "event": "save_api_key", "engine": engine, "key": mask_sensitive(&key) }));
    let app_data = app.path().app_local_data_dir().unwrap_or(PathBuf::from("."));
    SettingsService::new(&app_data).save_api_key(&engine, &key)
}

#[tauri::command]
pub async fn get_glossary(state: State<'_, AppState>) -> Result<Vec<GlossaryEntry>, String> {
    Ok(state.glossary_service.lock().await.get_all_entries())
}

#[tauri::command]
pub async fn add_glossary_entry(state: State<'_, AppState>, entry: GlossaryEntry) -> Result<(), String> {
    state.glossary_service.lock().await.add_entry(entry)
}

#[tauri::command]
pub async fn save_translation(
    mod_info: ModInfo,
    translations: Vec<TranslationItem>,
    target_lang: String,
) -> Result<(), String> {
    let ctx = LogContext::new("save_translation");
    info!("{}", json!({ "request_id": ctx.request_id, "mod": mod_info.name, "target_lang": target_lang }));

    let mut success_count = 0;

    for mut locale_file in mod_info.locale_files {
        // We only overwrite entries if we have a translation for them.
        for entry in &mut locale_file.entries {
            if let Some(t) = translations.iter().find(|t| t.section == entry.section && t.key == entry.key) {
                entry.value = t.translated_text.clone();
            }
        }
        
        let path = std::path::Path::new(&mod_info.source_path).join("locale").join(&target_lang).join(std::path::Path::new(&locale_file.file_path).file_name().unwrap());
        
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
            }
        }

        let file = std::fs::File::create(&path).map_err(|e| format!("Failed to create file {}: {}", path.display(), e))?;
        crate::services::cfg_parser::CfgParser::write(&locale_file, file).map_err(|e| format!("Failed to write cfg file {}: {}", path.display(), e))?;
        info!("{}", json!({ "request_id": ctx.request_id, "event": "file_saved", "path": path.display().to_string() }));
        success_count += 1;
    }

    info!("{}", json!({ "request_id": ctx.request_id, "event": "save_translation_completed", "saved_files": success_count }));
    ctx.complete();
    Ok(())
}

#[tauri::command]
pub async fn remove_glossary_entry(state: State<'_, AppState>, term: String) -> Result<(), String> {
    state.glossary_service.lock().await.remove_entry(&term)
}

#[tauri::command]
pub fn log_info(message: String) {
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&message) {
        info!("{}", json_val);
    } else {
        info!("{}", json!({ "message": message, "source": "frontend" }));
    }
}

#[tauri::command]
pub fn log_warn(message: String) {
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&message) {
        log::warn!("{}", json_val);
    } else {
        log::warn!("{}", json!({ "message": message, "source": "frontend" }));
    }
}

#[tauri::command]
pub fn log_error(message: String) {
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&message) {
        log::error!("{}", json_val);
    } else {
        log::error!("{}", json!({ "message": message, "source": "frontend" }));
    }
}

#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<TranslationRecord>, String> {
    state.history_service.get_all_history()
}

#[tauri::command]
pub async fn load_vanilla_data(state: State<'_, AppState>, factorio_path: String, lang_code: String) -> Result<(), String> {
    state.vanilla_service.lock().await.load_vanilla_data(&factorio_path, &lang_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use crate::models::cfg::{CfgFile, CfgEntry};
    use crate::models::enums::TranslationSource;

    #[tokio::test]
    async fn test_save_translation_writes_files_correctly() {
        let dir = tempdir().unwrap();
        let target_lang = "ja".to_string();
        
        // Mock ModInfo
        let mut mod_info = ModInfo {
            name: "test_mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Mod".to_string(),
            author: "Author".to_string(),
            source_path: dir.path().to_str().unwrap().to_string(),
            source_type: crate::models::enums::ModSourceType::Folder,
            factorio_version: None,
            locale_files: vec![
                CfgFile {
                    file_path: "locale/en/strings.cfg".to_string(),
                    language_code: "en".to_string(),
                    entries: vec![
                        CfgEntry {
                            section: "item-name".to_string(),
                            key: "iron-plate".to_string(),
                            value: "Iron Plate".to_string(),
                            comment: None,
                        }
                    ],
                    section_order: vec!["item-name".to_string()],
                    header_comments: vec![],
                }
            ],
        };

        let translations = vec![
            TranslationItem {
                section: "item-name".to_string(),
                key: "iron-plate".to_string(),
                source_text: "Iron Plate".to_string(),
                translated_text: "鉄板".to_string(),
                vanilla_translation: None,
                source: TranslationSource::Manual,
                is_edited: true,
            }
        ];

        let result = save_translation(mod_info, translations, target_lang.clone()).await;
        assert!(result.is_ok());

        // Verify file is saved
        let saved_file = dir.path().join("locale").join(&target_lang).join("strings.cfg");
        assert!(saved_file.exists());

        let content = fs::read_to_string(saved_file).unwrap();
        assert!(content.contains("[item-name]"));
        assert!(content.contains("iron-plate=鉄板"));
    }
}
