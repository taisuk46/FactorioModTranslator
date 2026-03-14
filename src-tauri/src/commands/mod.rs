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
use log::info;

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
    info!("Command: load_mod, path: {}", path);
    ModLoader::load_from_folder(&path)
        .or_else(|_| ModLoader::load_from_zip(&path))
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
    info!("Command: translate_mod, mod: {}, engine: {:?}", mod_info.title, engine_type);
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

    orchestrator.execute_translation(
        &mod_info,
        mode,
        &source_lang,
        &target_lang,
        |p| { let _ = app.emit("translation-progress", p); },
    ).await
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
pub async fn remove_glossary_entry(state: State<'_, AppState>, term: String) -> Result<(), String> {
    state.glossary_service.lock().await.remove_entry(&term)
}

#[tauri::command]
pub fn log_info(message: String) {
    info!("{}", message);
}

#[tauri::command]
pub fn log_warn(message: String) {
    log::warn!("{}", message);
}

#[tauri::command]
pub fn log_error(message: String) {
    log::error!("{}", message);
}

#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<TranslationRecord>, String> {
    state.history_service.get_all_history()
}

#[tauri::command]
pub async fn load_vanilla_data(state: State<'_, AppState>, factorio_path: String, lang_code: String) -> Result<(), String> {
    state.vanilla_service.lock().await.load_vanilla_data(&factorio_path, &lang_code)
}
