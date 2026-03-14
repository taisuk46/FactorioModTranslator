use std::path::{Path, PathBuf};
use rusqlite::{params, Connection};
use chrono::{DateTime, Utc};
use crate::models::translation::TranslationRecord;

pub struct TranslationHistoryService {
    db_path: PathBuf,
}

impl TranslationHistoryService {
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        if !app_data_dir.exists() {
            std::fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
        }
        let db_path = app_data_dir.join("history.db");
        let service = Self { db_path };
        service.initialize_database()?;
        Ok(service)
    }

    fn initialize_database(&self) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS translation_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mod_name TEXT NOT NULL,
                section TEXT NOT NULL,
                key TEXT NOT NULL,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                source_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                engine TEXT NOT NULL,
                translated_at TEXT NOT NULL,
                UNIQUE(mod_name, section, key, target_lang)
            );",
            [],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save_record(&self, record: &TranslationRecord) -> Result<(), String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO translation_history 
            (mod_name, section, key, source_lang, target_lang, source_text, translated_text, engine, translated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
            params![
                record.mod_name,
                record.section,
                record.key,
                record.source_lang,
                record.target_lang,
                record.source_text,
                record.translated_text,
                record.engine,
                record.translated_at.to_rfc3339(),
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_previous_translation(&self, mod_name: &str, section: &str, key: &str, target_lang: &str) -> Option<String> {
        let conn = Connection::open(&self.db_path).ok()?;
        let mut stmt = conn.prepare(
            "SELECT translated_text FROM translation_history 
             WHERE mod_name = ?1 AND section = ?2 AND key = ?3 AND target_lang = ?4 LIMIT 1;"
        ).ok()?;
        
        stmt.query_row(params![mod_name, section, key, target_lang], |row| row.get(0)).ok()
    }

    pub fn get_all_history(&self) -> Result<Vec<TranslationRecord>, String> {
        let conn = Connection::open(&self.db_path).map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, mod_name, section, key, source_lang, target_lang, source_text, translated_text, engine, translated_at 
             FROM translation_history ORDER BY translated_at DESC;"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |row| {
            let at_str: String = row.get(9)?;
            let translated_at = DateTime::parse_from_rfc3339(&at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(TranslationRecord {
                id: Some(row.get(0)?),
                mod_name: row.get(1)?,
                mod_version: None, // SQLite には保存していない
                section: row.get(2)?,
                key: row.get(3)?,
                source_lang: row.get(4)?,
                target_lang: row.get(5)?,
                source_text: row.get(6)?,
                translated_text: row.get(7)?,
                engine: row.get(8)?,
                translated_at,
            })
        }).map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| e.to_string())?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manage_history_success_should_add_and_get_history() {
        let dir = tempdir().unwrap();
        let service = TranslationHistoryService::new(dir.path()).unwrap();
        
        let record = TranslationRecord {
            id: None,
            mod_name: "test-mod".to_string(),
            mod_version: Some("1.0.0".to_string()),
            section: "item-name".to_string(),
            key: "iron-plate".to_string(),
            source_text: "Iron Plate".to_string(),
            translated_text: "鉄板".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            engine: "Google".to_string(),
            translated_at: chrono::Utc::now(),
        };

        service.save_record(&record).unwrap();
        let history = service.get_all_history().unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].translated_text, "鉄板");
    }

    #[test]
    fn test_manage_history_duplicate_should_not_create_duplicate_entries() {
        let dir = tempdir().unwrap();
        let service = TranslationHistoryService::new(dir.path()).unwrap();
        
        let record = TranslationRecord {
            id: None,
            mod_name: "test-mod".to_string(),
            mod_version: Some("1.0.0".to_string()),
            section: "item-name".to_string(),
            key: "iron-plate".to_string(),
            source_text: "Iron Plate".to_string(),
            translated_text: "鉄板".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            engine: "Google".to_string(),
            translated_at: chrono::Utc::now(),
        };

        service.save_record(&record).unwrap();
        service.save_record(&record).unwrap(); // Duplicate OR REPLACE
        let history = service.get_all_history().unwrap();

        assert_eq!(history.len(), 1);
    }
}
