use std::fs;
use std::path::{Path, PathBuf};
use serde_json::json;
use regex::{RegexBuilder, escape};
use crate::models::glossary::GlossaryEntry;
use log::{info, error};

pub struct GlossaryService {
    file_path: PathBuf,
    entries: Vec<GlossaryEntry>,
}

impl GlossaryService {
    pub fn new(app_data_dir: &Path) -> Self {
        let file_path = app_data_dir.join("glossary.json");
        let mut service = Self {
            file_path,
            entries: Vec::new(),
        };
        let _ = service.load();
        service
    }

    pub fn load(&mut self) -> Result<(), String> {
        if self.file_path.exists() {
            match fs::read_to_string(&self.file_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(entries) => {
                            self.entries = entries;
                            info!("{}", json!({ "event": "glossary_loaded", "count": self.entries.len(), "path": self.file_path.display().to_string() }));
                        }
                        Err(e) => {
                            error!("{}", json!({ "event": "glossary_parse_error", "error": e.to_string(), "path": self.file_path.display().to_string() }));
                            return Err(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    error!("{}", json!({ "event": "glossary_read_error", "error": e.to_string(), "path": self.file_path.display().to_string() }));
                    return Err(e.to_string());
                }
            }
        } else {
            info!("{}", json!({ "event": "glossary_not_found", "path": self.file_path.display().to_string(), "action": "created_empty" }));
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = self.file_path.parent().ok_or("Invalid glossary path")?;
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(&self.entries).map_err(|e| {
            error!("{}", json!({ "event": "glossary_serialize_error", "error": e.to_string() }));
            e.to_string()
        })?;
        
        fs::write(&self.file_path, json).map_err(|e| {
            error!("{}", json!({ "event": "glossary_write_error", "error": e.to_string(), "path": self.file_path.display().to_string() }));
            e.to_string()
        })?;
        
        info!("{}", json!({ "event": "glossary_saved", "count": self.entries.len(), "path": self.file_path.display().to_string() }));
        Ok(())
    }

    pub fn add_entry(&mut self, entry: GlossaryEntry) -> Result<(), String> {
        self.entries.retain(|e| {
            !(e.source_term == entry.source_term && e.source_lang == entry.source_lang && e.target_lang == entry.target_lang)
        });
        info!("{}", json!({ "event": "add_glossary_entry", "term": entry.source_term, "target": entry.target_term }));
        self.entries.push(entry);
        self.save()
    }

    pub fn remove_entry(&mut self, source_term: &str) -> Result<(), String> {
        let initial_len = self.entries.len();
        self.entries.retain(|e| e.source_term != source_term);
        
        if self.entries.len() < initial_len {
            info!("{}", json!({ "event": "remove_glossary_entry", "term": source_term, "removed": initial_len - self.entries.len() }));
        }
        
        self.save()
    }

    pub fn get_all_entries(&self) -> Vec<GlossaryEntry> {
        self.entries.clone()
    }

    pub fn apply_glossary(&self, text: &str, src_lang: &str, tgt_lang: &str) -> String {
        let mut result = text.to_string();
        for entry in self.entries.iter().filter(|e| e.source_lang == src_lang && e.target_lang == tgt_lang) {
            if entry.source_term.is_empty() {
                continue;
            }

            let pattern = escape(&entry.source_term);
            if let Ok(re) = RegexBuilder::new(&pattern).case_insensitive(true).build() {
                result = re.replace_all(&result, &entry.target_term).to_string();
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manage_glossary_success_should_replace_terms() {
        let dir = tempdir().unwrap();
        let mut service = GlossaryService::new(dir.path());
        let entry = GlossaryEntry { 
            source_term: "Iron".to_string(), 
            target_term: "鉄".to_string(), 
            source_lang: "en".to_string(), 
            target_lang: "ja".to_string(),
            exclude_from_translation: false, 
        };

        service.add_entry(entry).unwrap();
        let result = service.apply_glossary("Iron Plate", "en", "ja");

        assert_eq!(result, "鉄 Plate");
    }

    #[test]
    fn test_manage_glossary_alternative_duplicate_term_should_overwrite_existing() {
        let dir = tempdir().unwrap();
        let mut service = GlossaryService::new(dir.path());
        let entry1 = GlossaryEntry { 
            source_term: "Iron".to_string(), 
            target_term: "鉄".to_string(), 
            source_lang: "en".to_string(), 
            target_lang: "ja".to_string(),
            exclude_from_translation: false, 
        };
        let entry2 = GlossaryEntry { 
            source_term: "Iron".to_string(), 
            target_term: "アイアン".to_string(), 
            source_lang: "en".to_string(), 
            target_lang: "ja".to_string(),
            exclude_from_translation: false, 
        };

        service.add_entry(entry1).unwrap();
        service.add_entry(entry2).unwrap();
        let entries = service.get_all_entries();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].target_term, "アイアン");
    }

    #[test]
    fn test_manage_glossary_remove_entry() {
        let dir = tempdir().unwrap();
        let mut service = GlossaryService::new(dir.path());
        let entry = GlossaryEntry { 
            source_term: "Iron".to_string(), 
            target_term: "鉄".to_string(), 
            source_lang: "en".to_string(), 
            target_lang: "ja".to_string(),
            exclude_from_translation: false, 
        };

        service.add_entry(entry).unwrap();
        assert_eq!(service.get_all_entries().len(), 1);

        service.remove_entry("Iron").unwrap();
        // Ignore nonexistent removal too
        service.remove_entry("Copper").unwrap();
        
        let entries = service.get_all_entries();
        assert_eq!(entries.len(), 0);
    }
}
