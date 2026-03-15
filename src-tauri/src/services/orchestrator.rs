use std::path::Path;
use chrono::Utc;
use crate::models::mod_info::ModInfo;
use crate::models::enums::{TranslationMode, TranslationSource};
use crate::models::translation::{TranslationItem, TranslationRecord};
use crate::services::translation_engines::TranslationEngine;
use crate::services::vanilla_translation_service::VanillaTranslationService;
use crate::services::glossary_service::GlossaryService;
use crate::services::translation_history_service::TranslationHistoryService;
use log::{info, debug};
use serde_json::json;

pub struct TranslationOrchestrator<'a> {
    engine: &'a dyn TranslationEngine,
    vanilla: &'a VanillaTranslationService,
    glossary: &'a GlossaryService,
    history: &'a TranslationHistoryService,
}

impl<'a> TranslationOrchestrator<'a> {
    pub fn new(
        engine: &'a dyn TranslationEngine,
        vanilla: &'a VanillaTranslationService,
        glossary: &'a GlossaryService,
        history: &'a TranslationHistoryService,
    ) -> Self {
        Self {
            engine,
            vanilla,
            glossary,
            history,
        }
    }

    pub async fn execute_translation(
        &self,
        mod_info: &ModInfo,
        mode: TranslationMode,
        source_lang: &str,
        target_lang: &str,
        progress_callback: impl Fn(f64),
    ) -> Result<Vec<TranslationItem>, String> {
        info!("{}", json!({ "event": "execution_started", "mod": mod_info.name, "mode": format!("{:?}", mode) }));
        let mut results = Vec::new();
        let source_files: Vec<_> = mod_info.locale_files.iter()
            .filter(|f| f.language_code == source_lang)
            .collect();

        let target_files: std::collections::HashMap<_, _> = mod_info.locale_files.iter()
            .filter(|f| f.language_code == target_lang)
            .map(|f| (Path::new(&f.file_path).file_name().unwrap_or_default().to_string_lossy().to_string(), f))
            .collect();

        let total_entries: usize = source_files.iter().map(|f| f.entries.len()).sum();
        let mut processed_count = 0;

        let mut counts = std::collections::HashMap::new();

        for cfg_file in source_files {
            debug!("{}", json!({ "event": "processing_file", "file": cfg_file.file_path }));
            let file_name = Path::new(&cfg_file.file_path).file_name().unwrap_or_default().to_string_lossy().to_string();
            let target_file = target_files.get(&file_name);

            for entry in &cfg_file.entries {
                let mut item = TranslationItem {
                    section: entry.section.clone(),
                    key: entry.key.clone(),
                    source_text: entry.value.clone(),
                    translated_text: String::new(),
                    vanilla_translation: None,
                    source: TranslationSource::API, // Default
                    is_edited: false,
                };

                // 1. Check if we should skip (Diff mode)
                if mode == TranslationMode::DiffTranslation {
                    if let Some(tf) = target_file {
                        if let Some(existing) = tf.entries.iter().find(|e| e.section == entry.section && e.key == entry.key) {
                            if !existing.value.is_empty() {
                                item.translated_text = existing.value.clone();
                                item.source = TranslationSource::History;
                                results.push(item);
                                processed_count += 1;
                                progress_callback(processed_count as f64 / total_entries as f64);
                                *counts.entry("skipped").or_insert(0) += 1;
                                continue;
                            }
                        }
                    }
                }

                // 2. Vanilla Match
                if let Some(vanilla_match) = self.vanilla.match_by_key(&entry.section, &entry.key) {
                    item.translated_text = vanilla_match.clone();
                    item.vanilla_translation = Some(vanilla_match);
                    item.source = TranslationSource::VanillaKeyMatch;
                } else if let Some(vanilla_match) = self.vanilla.match_by_text(&entry.value) {
                    item.translated_text = vanilla_match.clone();
                    item.vanilla_translation = Some(vanilla_match);
                    item.source = TranslationSource::VanillaTextMatch;
                }

                // 3. History Match (if no vanilla match)
                if item.translated_text.is_empty() {
                    if let Some(history_match) = self.history.get_previous_translation(&mod_info.name, &entry.section, &entry.key, target_lang) {
                        item.translated_text = history_match;
                        item.source = TranslationSource::History;
                    }
                }

                // 4. API Translation (if still no translation or Overwrite mode)
                if item.translated_text.is_empty() || mode == TranslationMode::OverwriteUpdate {
                    // Apply glossary first
                    let text_to_translate = self.glossary.apply_glossary(&entry.value, source_lang, target_lang);
                    
                    // Execute API call
                    match self.engine.translate(&text_to_translate, source_lang, target_lang).await {
                        Ok(translated) => {
                            item.translated_text = translated;
                            item.source = TranslationSource::API;
                        }
                        Err(e) => {
                            log::error!("{}", json!({ "event": "api_error", "error": e, "key": format!("{}.{}", entry.section, entry.key) }));
                            return Err(format!("Translation failed: {}", e));
                        }
                    }
                }

                *counts.entry(match item.source {
                    TranslationSource::API => "api",
                    TranslationSource::VanillaKeyMatch | TranslationSource::VanillaTextMatch => "vanilla",
                    TranslationSource::History => "history",
                    TranslationSource::Glossary => "glossary",
                    TranslationSource::Manual => "manual",
                }).or_insert(0) += 1;

                results.push(item.clone());
                
                // Save to history
                let record = TranslationRecord {
                    id: None,
                    mod_name: mod_info.name.clone(),
                    mod_version: Some(mod_info.version.clone()),
                    section: entry.section.clone(),
                    key: entry.key.clone(),
                    source_lang: source_lang.to_string(),
                    target_lang: target_lang.to_string(),
                    source_text: entry.value.clone(),
                    translated_text: item.translated_text.clone(),
                    engine: self.engine.name().to_string(),
                    translated_at: Utc::now(),
                };
                let _ = self.history.save_record(&record);

                processed_count += 1;
                progress_callback(processed_count as f64 / total_entries as f64);
            }
        }

        info!("{}", json!({ "event": "execution_completed", "summary": counts }));
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use tempfile::tempdir;
    use crate::models::cfg::CfgFile;
    use crate::models::mod_info::ModInfo;
    use crate::models::glossary::GlossaryEntry;

    struct MockEngine;
    #[async_trait]
    impl TranslationEngine for MockEngine {
        async fn translate(&self, text: &str, _: &str, _: &str) -> Result<String, String> {
            Ok(format!("Mock: {}", text))
        }
        async fn translate_batch(&self, texts: Vec<String>, _: &str, _: &str) -> Result<Vec<String>, String> {
            Ok(texts.into_iter().map(|t| format!("Mock: {}", t)).collect())
        }
        fn name(&self) -> &str { "Mock" }
    }

    #[tokio::test]
    async fn test_orchestrator_should_prefer_vanilla_match() {
        let dir = tempdir().unwrap();
        let engine = MockEngine;
        let mut vanilla = VanillaTranslationService::new();
        // Mock data path for vanilla
        let data_dir = dir.path().join("data");
        std::fs::create_dir_all(data_dir.join("core/locale/en")).unwrap();
        std::fs::write(data_dir.join("core/locale/en/core.cfg"), "[gui]\nconfirm=Confirm").unwrap();
        std::fs::create_dir_all(data_dir.join("core/locale/ja")).unwrap();
        std::fs::write(data_dir.join("core/locale/ja/core.cfg"), "[gui]\nconfirm=確定").unwrap();
        vanilla.load_vanilla_data(&dir.path().to_string_lossy(), "ja").unwrap();

        let glossary = GlossaryService::new(dir.path());
        let history = TranslationHistoryService::new(dir.path()).unwrap();
        
        let orchestrator = TranslationOrchestrator::new(&engine, &vanilla, &glossary, &history);
        
        let mod_info = ModInfo {
            name: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Mod".to_string(),
            author: "Author".to_string(),
            source_path: String::new(),
            source_type: crate::models::enums::ModSourceType::Folder,
            factorio_version: None,
            locale_files: vec![
                CfgFile {
                    file_path: "locale/en/test.cfg".to_string(),
                    language_code: "en".to_string(),
                    entries: vec![
                        crate::models::cfg::CfgEntry {
                            section: "gui".to_string(),
                            key: "confirm".to_string(),
                            value: "Confirm".to_string(),
                            comment: None,
                        }
                    ],
                    section_order: vec![],
                    header_comments: vec![],
                }
            ],
        };

        let results = orchestrator.execute_translation(&mod_info, TranslationMode::NewTranslation, "en", "ja", |_| {}).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].translated_text, "確定");
        assert_eq!(results[0].source, TranslationSource::VanillaKeyMatch);
    }

    #[tokio::test]
    async fn test_orchestrator_should_apply_glossary_before_api() {
        let dir = tempdir().unwrap();
        let engine = MockEngine;
        let vanilla = VanillaTranslationService::new();
        let mut glossary = GlossaryService::new(dir.path());
        glossary.add_entry(GlossaryEntry {
            source_term: "Iron".to_string(),
            target_term: "鉄".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            exclude_from_translation: false,
        }).unwrap();
        let history = TranslationHistoryService::new(dir.path()).unwrap();
        
        let orchestrator = TranslationOrchestrator::new(&engine, &vanilla, &glossary, &history);
        
        let mod_info = ModInfo {
            name: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Mod".to_string(),
            author: "Author".to_string(),
            source_path: String::new(),
            source_type: crate::models::enums::ModSourceType::Folder,
            factorio_version: None,
            locale_files: vec![
                CfgFile {
                    file_path: "locale/en/test.cfg".to_string(),
                    language_code: "en".to_string(),
                    entries: vec![
                        crate::models::cfg::CfgEntry {
                            section: "item-name".to_string(),
                            key: "iron-plate".to_string(),
                            value: "Iron Plate".to_string(),
                            comment: None,
                        }
                    ],
                    section_order: vec![],
                    header_comments: vec![],
                }
            ],
        };

        let results = orchestrator.execute_translation(&mod_info, TranslationMode::NewTranslation, "en", "ja", |_| {}).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].translated_text, "Mock: 鉄 Plate");
        assert_eq!(results[0].source, TranslationSource::API);
    }

    #[tokio::test]
    async fn test_orchestrator_diff_translation_skips_existing() {
        let dir = tempdir().unwrap();
        let engine = MockEngine;
        let vanilla = VanillaTranslationService::new();
        let glossary = GlossaryService::new(dir.path());
        let history = TranslationHistoryService::new(dir.path()).unwrap();
        
        let orchestrator = TranslationOrchestrator::new(&engine, &vanilla, &glossary, &history);
        
        let mod_info = ModInfo {
            name: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test Mod".to_string(),
            author: "Author".to_string(),
            source_path: String::new(),
            source_type: crate::models::enums::ModSourceType::Folder,
            factorio_version: None,
            locale_files: vec![
                CfgFile {
                    file_path: "locale/ja/test.cfg".to_string(),
                    language_code: "ja".to_string(),
                    entries: vec![
                        crate::models::cfg::CfgEntry {
                            section: "item-name".to_string(),
                            key: "iron-plate".to_string(),
                            value: "鉄の板".to_string(),
                            comment: None,
                        }
                    ],
                    section_order: vec![],
                    header_comments: vec![],
                }
            ],
        };

        // If target lang is 'ja' and it already exists in a file with language 'ja', Diff should skip it
        let results = orchestrator.execute_translation(&mod_info, TranslationMode::DiffTranslation, "en", "ja", |_| {}).await.unwrap();
        
        assert_eq!(results.len(), 0); // Should be omitted because it's DiffTranslation and already exists
    }

    #[tokio::test]
    async fn test_orchestrator_history_match() {
        let dir = tempdir().unwrap();
        let engine = MockEngine;
        let vanilla = VanillaTranslationService::new();
        let glossary = GlossaryService::new(dir.path());
        let history = TranslationHistoryService::new(dir.path()).unwrap();
        
        let record = crate::models::translation::TranslationRecord {
            id: Some(1),
            mod_name: "test-mod".to_string(),
            mod_version: Some("1.0.0".to_string()),
            section: "item-name".to_string(),
            key: "copper-plate".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            source_text: "Copper Plate".to_string(),
            translated_text: "銅板(History)".to_string(),
            engine: "Mock".to_string(),
            translated_at: chrono::Utc::now(),
        };
        history.save_record(&record).unwrap();

        let orchestrator = TranslationOrchestrator::new(&engine, &vanilla, &glossary, &history);
        
        let mod_info = ModInfo {
            name: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test".to_string(),
            author: "A".to_string(),
            source_path: String::new(),
            source_type: crate::models::enums::ModSourceType::Folder,
            factorio_version: None,
            locale_files: vec![
                CfgFile {
                    file_path: "locale/en/test.cfg".to_string(),
                    language_code: "en".to_string(),
                    entries: vec![
                        crate::models::cfg::CfgEntry {
                            section: "item-name".to_string(),
                            key: "copper-plate".to_string(),
                            value: "Copper Plate".to_string(),
                            comment: None,
                        }
                    ],
                    section_order: vec![],
                    header_comments: vec![],
                }
            ],
        };

        let results = orchestrator.execute_translation(&mod_info, TranslationMode::NewTranslation, "en", "ja", |_| {}).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].translated_text, "銅板(History)");
        assert_eq!(results[0].source, TranslationSource::History);
    }

    struct ErrorEngine;
    #[async_trait]
    impl TranslationEngine for ErrorEngine {
        async fn translate(&self, _: &str, _: &str, _: &str) -> Result<String, String> {
            Err("API Limit Exceeded".to_string())
        }
        async fn translate_batch(&self, _: Vec<String>, _: &str, _: &str) -> Result<Vec<String>, String> {
            Err("API Limit Exceeded".to_string())
        }
        fn name(&self) -> &str { "ErrorMock" }
    }

    #[tokio::test]
    async fn test_orchestrator_api_error_handling() {
        let dir = tempdir().unwrap();
        let engine = ErrorEngine;
        let vanilla = VanillaTranslationService::new();
        let glossary = GlossaryService::new(dir.path());
        let history = TranslationHistoryService::new(dir.path()).unwrap();
        
        let orchestrator = TranslationOrchestrator::new(&engine, &vanilla, &glossary, &history);
        
        let mod_info = ModInfo {
            name: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            title: "Test".to_string(),
            author: "A".to_string(),
            source_path: String::new(),
            source_type: crate::models::enums::ModSourceType::Folder,
            factorio_version: None,
            locale_files: vec![
                CfgFile {
                    file_path: "locale/en/test.cfg".to_string(),
                    language_code: "en".to_string(),
                    entries: vec![
                        crate::models::cfg::CfgEntry {
                            section: "item-name".to_string(),
                            key: "copper-plate".to_string(),
                            value: "Copper Plate".to_string(),
                            comment: None,
                        }
                    ],
                    section_order: vec![],
                    header_comments: vec![],
                }
            ],
        };

        // The orchestrator returns Err if translate_batch fails.
        let result = orchestrator.execute_translation(&mod_info, TranslationMode::NewTranslation, "en", "ja", |_| {}).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API Limit Exceeded"));
    }
}
