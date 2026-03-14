pub mod models;
pub mod services;
pub mod commands;

use tauri::Manager;
use crate::commands::{AppState, load_mod, translate_mod, get_settings, save_settings, save_api_key, get_glossary, add_glossary_entry, remove_glossary_entry, get_history, load_vanilla_data, get_localized_strings, log_info, log_warn, log_error};
use crate::services::vanilla_translation_service::VanillaTranslationService;
use crate::services::glossary_service::GlossaryService;
use crate::services::translation_history_service::TranslationHistoryService;
use crate::services::localization_service::LocalizationService;

use log::info;
use tauri_plugin_log::{Target, TargetKind};
use chrono::Local;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    info!("Starting Factorio Mod Translator backend...");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::Folder {
                    path: std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string())).join("FactorioModTranslator").join("logs"),
                    file_name: Some(format!("log_{}", Local::now().format("%Y%m%d"))),
                }),
            ])
            .format(|out, message, record| {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S.%3f");
                let level = record.level();
                let file = record.file().unwrap_or("unknown");
                let line = record.line().unwrap_or(0);
                let file_name = std::path::Path::new(file).file_name().and_then(|n| n.to_str()).unwrap_or(file);
                
                out.finish(format_args!(
                    "[{}] [{}] [{}:{}] {}",
                    timestamp, level, file_name, line, message
                ))
            })
            .level(log::LevelFilter::Info)
            .build())
        .setup(|app| {
            info!("Backend services initializing...");
            let app_data = app.path().app_local_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            info!("App local data dir: {:?}", app_data);
            let state = AppState {
                vanilla_service: tokio::sync::Mutex::new(VanillaTranslationService::new()),
                glossary_service: tokio::sync::Mutex::new(GlossaryService::new(&app_data)),
                history_service: TranslationHistoryService::new(&app_data).expect("Failed to init history"),
                localization_service: LocalizationService::new(),
            };
            app.manage(state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_mod,
            translate_mod,
            get_settings,
            save_settings,
            save_api_key,
            get_glossary,
            add_glossary_entry,
            remove_glossary_entry,
            get_history,
            load_vanilla_data,
            get_localized_strings,
            log_info,
            log_warn,
            log_error
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
