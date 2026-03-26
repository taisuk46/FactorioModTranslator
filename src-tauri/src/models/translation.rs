use crate::models::enums::TranslationSource;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_record_creation() {
        let record = TranslationRecord {
            id: Some(1),
            mod_name: "test-mod".to_string(),
            mod_version: Some("1.0.0".to_string()),
            section: "item-name".to_string(),
            key: "iron-plate".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            source_text: "Iron Plate".to_string(),
            translated_text: "鉄板".to_string(),
            engine: "DeepL".to_string(),
            translated_at: Utc::now(),
        };

        assert_eq!(record.id, Some(1));
        assert_eq!(record.mod_name, "test-mod");
        assert_eq!(record.mod_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_translation_record_without_id() {
        let record = TranslationRecord {
            id: None,
            mod_name: "test-mod".to_string(),
            mod_version: None,
            section: "item-name".to_string(),
            key: "copper-plate".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            source_text: "Copper Plate".to_string(),
            translated_text: "銅板".to_string(),
            engine: "Google".to_string(),
            translated_at: Utc::now(),
        };

        assert!(record.id.is_none());
        assert!(record.mod_version.is_none());
    }

    #[test]
    fn test_translation_item_creation() {
        let item = TranslationItem {
            section: "item-name".to_string(),
            key: "iron-plate".to_string(),
            source_text: "Iron Plate".to_string(),
            translated_text: "鉄板".to_string(),
            vanilla_translation: Some("鉄板".to_string()),
            source: TranslationSource::VanillaKeyMatch,
            is_edited: false,
        };

        assert_eq!(item.section, "item-name");
        assert_eq!(item.key, "iron-plate");
        assert_eq!(item.source, TranslationSource::VanillaKeyMatch);
        assert!(!item.is_edited);
    }

    #[test]
    fn test_translation_item_api_source() {
        let item = TranslationItem {
            section: "recipe-name".to_string(),
            key: "iron-gear".to_string(),
            source_text: "Iron Gear".to_string(),
            translated_text: "鉄歯車".to_string(),
            vanilla_translation: None,
            source: TranslationSource::API,
            is_edited: true,
        };

        assert_eq!(item.source, TranslationSource::API);
        assert!(item.is_edited);
    }

    #[test]
    fn test_translation_item_serialization() {
        let item = TranslationItem {
            section: "item-name".to_string(),
            key: "iron-plate".to_string(),
            source_text: "Iron Plate".to_string(),
            translated_text: "鉄板".to_string(),
            vanilla_translation: None,
            source: TranslationSource::Manual,
            is_edited: false,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("item-name"));
        assert!(json.contains("Manual"));

        let deserialized: TranslationItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.key, "iron-plate");
    }
}
