use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::enums::TranslationSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRecord {
    pub id: Option<i64>,
    pub mod_name: String,
    pub mod_version: Option<String>,
    pub section: String,
    pub key: String,
    pub source_lang: String,
    pub target_lang: String,
    pub source_text: String,
    pub translated_text: String,
    pub engine: String,
    pub translated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationItem {
    pub section: String,
    pub key: String,
    pub source_text: String,
    pub translated_text: String,
    pub vanilla_translation: Option<String>,
    pub source: TranslationSource,
    pub is_edited: bool,
}
