use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub source_term: String,
    pub target_term: String,
    pub source_lang: String,
    pub target_lang: String,
    pub exclude_from_translation: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossary_entry_creation() {
        let entry = GlossaryEntry {
            source_term: "Iron Plate".to_string(),
            target_term: "鉄板".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            exclude_from_translation: false,
        };

        assert_eq!(entry.source_term, "Iron Plate");
        assert_eq!(entry.target_term, "鉄板");
        assert_eq!(entry.source_lang, "en");
        assert_eq!(entry.target_lang, "ja");
    }

    #[test]
    fn test_glossary_entry_exclude_from_translation() {
        let entry = GlossaryEntry {
            source_term: "Item".to_string(),
            target_term: "アイテム".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            exclude_from_translation: true,
        };

        assert!(entry.exclude_from_translation);
    }

    #[test]
    fn test_glossary_entry_serialization() {
        let entry = GlossaryEntry {
            source_term: "Copper".to_string(),
            target_term: "銅".to_string(),
            source_lang: "en".to_string(),
            target_lang: "ja".to_string(),
            exclude_from_translation: false,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Copper"));
        assert!(json.contains("銅"));

        let deserialized: GlossaryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source_term, "Copper");
    }

    #[test]
    fn test_glossary_entry_default_values() {
        // Test with empty strings
        let entry = GlossaryEntry {
            source_term: String::new(),
            target_term: String::new(),
            source_lang: String::new(),
            target_lang: String::new(),
            exclude_from_translation: false,
        };

        assert!(entry.source_term.is_empty());
        assert!(entry.target_term.is_empty());
    }
}
