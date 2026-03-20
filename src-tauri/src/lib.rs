pub mod models;
pub mod services;
pub mod commands;

use tauri::Manager;
use crate::commands::{AppState, load_mod, translate_mod, get_settings, save_settings, save_api_key, get_glossary, add_glossary_entry, remove_glossary_entry, get_history, load_vanilla_data, get_localized_strings, log_info, log_warn, log_error, save_translation, select_mod_path};
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
                Target::new(TargetKind::LogDir { file_name: None }),
            ])
            .format(|out, message, record| {
                let timestamp = Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                let level = record.level().to_string();
                let file = record.file().unwrap_or("unknown");
                let line = record.line().unwrap_or(0);
                let target = record.target();
                
                let msg_str = format!("{}", message);
                // Attempt to parse message as JSON to flatten it
                let msg_json: serde_json::Value = serde_json::from_str(&msg_str).unwrap_or(serde_json::Value::String(msg_str));

                let mut log_obj = serde_json::json!({
                    "timestamp": timestamp,
                    "level": level,
                    "target": target,
                    "file": file,
                    "line": line,
                });

                if let serde_json::Value::Object(map) = msg_json {
                    if let serde_json::Value::Object(ref mut log_map) = log_obj {
                        for (k, v) in map {
                            log_map.insert(k, v);
                        }
                    }
                } else {
                    log_obj["message"] = msg_json;
                }

                out.finish(format_args!("{}", log_obj))
            })
            .level(log::LevelFilter::Debug)
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
            log_error,
            save_translation,
            select_mod_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
