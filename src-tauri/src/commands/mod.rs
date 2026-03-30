use tauri::{AppHandle, Manager, State, Emitter};
use std::io::Read;
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
use log::{info, error};
use serde_json::json;

pub struct AppState {
    pub vanilla_service: tokio::sync::Mutex<VanillaTranslationService>,
    pub glossary_service: tokio::sync::Mutex<GlossaryService>,
    pub history_service: TranslationHistoryService,
    pub localization_service: LocalizationService,
}

#[tauri::command]
pub async fn select_mod_path() -> Result<Option<String>, String> {
    let res = rfd::FileDialog::new()
        .set_title("Select Factorio Mod Folder")
        .pick_folder();
    
    Ok(res.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn select_mod_zip_path() -> Result<Option<String>, String> {
    let res = rfd::FileDialog::new()
        .set_title("Select Factorio Mod ZIP File")
        .add_filter("ZIP files", &["zip"])
        .pick_file();
    
    Ok(res.map(|p| p.to_string_lossy().to_string()))
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
pub async fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    info!("Command: get_settings");
    let app_data = app.path().app_local_data_dir().unwrap_or(PathBuf::from("."));
    Ok(SettingsService::new(&app_data).load_settings())
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

    if mod_info.source_type == crate::models::enums::ModSourceType::Zip {
        // ZIPソースの場合、保存形式を選択
        let save_result = rfd::MessageDialog::new()
            .set_title("保存形式の選択")
            .set_description("ZIPファイルから読み込まれたModです。\n「はい」= フォルダに保存\n「いいえ」= ZIPに保存")
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        match save_result {
            rfd::MessageDialogResult::Yes => {
                // フォルダ保存
                let save_dir = rfd::FileDialog::new()
                    .set_title("保存先フォルダを選択")
                    .pick_folder()
                    .ok_or("保存先が選択されませんでした")?;
                
                save_to_folder(&mod_info, &translations, &target_lang, save_dir.to_str().unwrap(), &ctx)?;
            }
            rfd::MessageDialogResult::No => {
                // ZIP保存
                let save_path = rfd::FileDialog::new()
                    .set_title("保存先ZIPファイルを選択")
                    .add_filter("ZIP files", &["zip"])
                    .save_file()
                    .ok_or("保存先が選択されませんでした")?;
                
                save_to_zip(&mod_info, &translations, &target_lang, save_path.to_str().unwrap(), &ctx)?;
            }
            _ => {
                return Err("保存がキャンセルされました".to_string());
            }
        }
    } else {
        // フォルダソースの場合、既存の処理
        save_to_folder(&mod_info, &translations, &target_lang, &mod_info.source_path, &ctx)?;
    }

    ctx.complete();
    Ok(())
}

fn save_to_folder(
    mod_info: &ModInfo,
    translations: &[TranslationItem],
    target_lang: &str,
    base_path: &str,
    ctx: &LogContext,
) -> Result<(), String> {
    let mut success_count = 0;
    let mut total_files = 0;

    for locale_file in &mod_info.locale_files {
        total_files += 1;
        let mut entries_updated = 0;
        let mut updated_entries = locale_file.entries.clone();
        
        for entry in &mut updated_entries {
            if let Some(t) = translations.iter().find(|t| t.section == entry.section && t.key == entry.key) {
                entry.value = t.translated_text.clone();
                entries_updated += 1;
            }
        }
        
        if entries_updated == 0 {
            continue;
        }

        let file_name = std::path::Path::new(&locale_file.file_path)
            .file_name()
            .ok_or_else(|| "Invalid file path".to_string())?;
        let path = std::path::Path::new(base_path)
            .join("locale")
            .join(target_lang)
            .join(file_name);
        
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
            }
        }

        let cfg_file = crate::models::cfg::CfgFile {
            file_path: locale_file.file_path.clone(),
            language_code: target_lang.to_string(),
            entries: updated_entries,
            section_order: locale_file.section_order.clone(),
            header_comments: locale_file.header_comments.clone(),
        };

        let file = std::fs::File::create(&path)
            .map_err(|e| format!("Failed to create file {}: {}", path.display(), e))?;
        crate::services::cfg_parser::CfgParser::write(&cfg_file, file)
            .map_err(|e| format!("Failed to write cfg file {}: {}", path.display(), e))?;
        
        info!("{}", json!({ "request_id": ctx.request_id, "event": "file_saved", "path": path.display().to_string() }));
        success_count += 1;
    }

    if success_count == 0 && total_files > 0 {
        return Err("No translations were found to save in any of the locale files.".to_string());
    }

    info!("{}", json!({ "request_id": ctx.request_id, "event": "save_translation_completed", "saved_files": success_count }));
    Ok(())
}

fn save_to_zip(
    mod_info: &ModInfo,
    translations: &[TranslationItem],
    target_lang: &str,
    zip_path: &str,
    ctx: &LogContext,
) -> Result<(), String> {
    // 元のZIPを開く
    let src_file = std::fs::File::open(&mod_info.source_path)
        .map_err(|e| format!("Failed to open source ZIP: {}", e))?;
    let mut src_archive = zip::ZipArchive::new(src_file)
        .map_err(|e| format!("Failed to read source ZIP: {}", e))?;

    // 新しいZIPを作成
    let dst_file = std::fs::File::create(zip_path)
        .map_err(|e| format!("Failed to create destination ZIP: {}", e))?;
    let mut dst_zip = zip::ZipWriter::new(dst_file);

    let options: zip::write::FileOptions<()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Factorio modsのルートフォルダを取得
    let first_entry_name = src_archive.by_index(0)
        .map_err(|e| e.to_string())?
        .name().to_string();
    let root_folder = first_entry_name.split('/').next().unwrap_or_default().to_string() + "/";

    // 翻訳済みエントリのマップを作成
    let mut translation_map: std::collections::HashMap<String, &TranslationItem> = std::collections::HashMap::new();
    for t in translations {
        let key = format!("{}.{}", t.section, t.key);
        translation_map.insert(key, t);
    }

    // ソース言語 (locale_filesの最初の言語) を特定
    let source_lang = mod_info.locale_files.first()
        .map(|f| f.language_code.clone())
        .unwrap_or_else(|| "en".to_string());
    let source_locale_prefix = format!("{}locale/{}/", root_folder, source_lang);
    let target_locale_prefix = format!("{}locale/{}/", root_folder, target_lang);

    // 全エントリをコピーし、ソース言語の.cfgは対象言語に変換して追加
    for i in 0..src_archive.len() {
        let mut entry = src_archive.by_index(i)
            .map_err(|e| e.to_string())?;
        let entry_name = entry.name().to_string();

        // ソース言語の.cfgファイルかチェック
        if entry_name.starts_with(&source_locale_prefix) && entry_name.ends_with(".cfg") {
            // 対応するlocale_fileを探す
            if let Some(locale_file) = mod_info.locale_files.iter().find(|f| f.file_path == entry_name) {
                // 翻訳済みのエントリで上書き
                let mut updated_entries = locale_file.entries.clone();
                for entry_item in &mut updated_entries {
                    let key = format!("{}.{}", entry_item.section, entry_item.key);
                    if let Some(t) = translation_map.get(&key) {
                        entry_item.value = t.translated_text.clone();
                    }
                }

                let cfg_file = crate::models::cfg::CfgFile {
                    file_path: entry_name.clone(),
                    language_code: target_lang.to_string(),
                    entries: updated_entries,
                    section_order: locale_file.section_order.clone(),
                    header_comments: locale_file.header_comments.clone(),
                };

                // 新しいターゲット言語のファイルパスを作成
                let target_file_name = entry_name.replace(&source_locale_prefix, &target_locale_prefix);
                dst_zip.start_file(&target_file_name, options)
                    .map_err(|e| format!("Failed to start file in ZIP: {}", e))?;
                let mut cursor = std::io::Cursor::new(Vec::new());
                crate::services::cfg_parser::CfgParser::write(&cfg_file, &mut cursor)
                    .map_err(|e| format!("Failed to write cfg to ZIP: {}", e))?;
                std::io::Write::write_all(&mut dst_zip, &cursor.into_inner())
                    .map_err(|e| format!("Failed to write to ZIP: {}", e))?;
            } else {
                // 対応するlocale_fileが見つからない場合はそのままコピー
                dst_zip.start_file(&entry_name, options)
                    .map_err(|e| format!("Failed to start file in ZIP: {}", e))?;
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)
                    .map_err(|e| format!("Failed to read entry: {}", e))?;
                std::io::Write::write_all(&mut dst_zip, &buf)
                    .map_err(|e| format!("Failed to write to ZIP: {}", e))?;
            }
        } else {
            // その他のエントリはそのままコピー
            dst_zip.start_file(&entry_name, options)
                .map_err(|e| format!("Failed to start file in ZIP: {}", e))?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)
                .map_err(|e| format!("Failed to read entry: {}", e))?;
            std::io::Write::write_all(&mut dst_zip, &buf)
                .map_err(|e| format!("Failed to write to ZIP: {}", e))?;
        }
    }

    dst_zip.finish()
        .map_err(|e| format!("Failed to finalize ZIP: {}", e))?;

    info!("{}", json!({ "request_id": ctx.request_id, "event": "zip_saved", "path": zip_path }));
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
    use std::io::Write as _;
    use crate::models::cfg::{CfgFile, CfgEntry};
    use crate::models::enums::TranslationSource;
    use crate::services::logging::LogContext;

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

    #[tokio::test]
    async fn test_save_to_folder_creates_locale_structure() {
        let dir = tempdir().unwrap();
        let target_lang = "ja".to_string();
        
        let mod_info = ModInfo {
            name: "test_mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Mod".to_string(),
            author: "Author".to_string(),
            source_path: "/dummy/path".to_string(),
            source_type: crate::models::enums::ModSourceType::Zip,
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

        let ctx = LogContext::new("test");
        let result = save_to_folder(&mod_info, &translations, &target_lang, dir.path().to_str().unwrap(), &ctx);
        assert!(result.is_ok());

        let saved_file = dir.path().join("locale").join(&target_lang).join("strings.cfg");
        assert!(saved_file.exists());

        let content = fs::read_to_string(saved_file).unwrap();
        assert!(content.contains("[item-name]"));
        assert!(content.contains("iron-plate=鉄板"));
    }

    #[tokio::test]
    async fn test_save_to_folder_no_matching_translations() {
        let dir = tempdir().unwrap();
        let target_lang = "ja".to_string();
        
        let mod_info = ModInfo {
            name: "test_mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Mod".to_string(),
            author: "Author".to_string(),
            source_path: "/dummy/path".to_string(),
            source_type: crate::models::enums::ModSourceType::Zip,
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

        // 翻訳に一致するエントリなし
        let translations = vec![
            TranslationItem {
                section: "other-section".to_string(),
                key: "other-key".to_string(),
                source_text: "Other".to_string(),
                translated_text: "他".to_string(),
                vanilla_translation: None,
                source: TranslationSource::Manual,
                is_edited: true,
            }
        ];

        let ctx = LogContext::new("test");
        let result = save_to_folder(&mod_info, &translations, &target_lang, dir.path().to_str().unwrap(), &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No translations were found"));
    }

    #[tokio::test]
    async fn test_save_to_zip_creates_translated_zip() {
        let dir = tempdir().unwrap();
        
        // テスト用のZIPファイルを作成
        let source_zip_path = dir.path().join("test-mod_1.0.0.zip");
        let source_file = fs::File::create(&source_zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(source_file);
        let options: zip::write::FileOptions<()> = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("test-mod_1.0.0/info.json", options).unwrap();
        zip.write_all(b"{\"name\": \"test-mod\", \"version\": \"1.0.0\"}").unwrap();

        zip.start_file("test-mod_1.0.0/locale/en/strings.cfg", options).unwrap();
        zip.write_all(b"[item-name]\niron-plate=Iron Plate\n").unwrap();

        zip.finish().unwrap();

        // ModLoaderで読み込み
        let mut mod_info = crate::services::mod_loader::ModLoader::load_from_zip(source_zip_path.to_str().unwrap()).unwrap();
        mod_info.source_path = source_zip_path.to_str().unwrap().to_string();

        let translations = vec![
            TranslationItem {
                section: "item-name".to_string(),
                key: "iron-plate".to_string(),
                source_text: "Iron Plate".to_string(),
                translated_text: "鉄板".to_string(),
                vanilla_translation: None,
                source: TranslationSource::API,
                is_edited: false,
            }
        ];

        let output_zip_path = dir.path().join("test-mod_ja.zip");
        let ctx = LogContext::new("test");
        let result = save_to_zip(&mod_info, &translations, "ja", output_zip_path.to_str().unwrap(), &ctx);
        assert!(result.is_ok());
        assert!(output_zip_path.exists());

        // 出力ZIPを検証
        let output_file = fs::File::open(&output_zip_path).unwrap();
        let mut output_archive = zip::ZipArchive::new(output_file).unwrap();
        
        // locale/ja/strings.cfgが含まれているか確認
        let mut ja_cfg = output_archive.by_name("test-mod_1.0.0/locale/ja/strings.cfg").unwrap();
        let mut content = String::new();
        ja_cfg.read_to_string(&mut content).unwrap();
        assert!(content.contains("iron-plate=鉄板"));
    }
}
