use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModSourceType {
    Folder,
    Zip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TranslationMode {
    NewTranslation,
    DiffTranslation,
    OverwriteUpdate,
    ManualEdit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TranslationEngineType {
    #[default]
    DeepL,
    GoogleTranslate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TranslationSource {
    API,
    VanillaKeyMatch,
    VanillaTextMatch,
    Manual,
    History,
    Glossary,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_mod_source_type_serialization() {
        let folder = ModSourceType::Folder;
        let json = serde_json::to_string(&folder).unwrap();
        assert_eq!(json, "\"Folder\"");

        let deserialized: ModSourceType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ModSourceType::Folder);
    }

    #[test]
    fn test_mod_source_type_zip_serialization() {
        let zip = ModSourceType::Zip;
        let json = serde_json::to_string(&zip).unwrap();
        assert_eq!(json, "\"Zip\"");

        let deserialized: ModSourceType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ModSourceType::Zip);
    }

    #[test]
    fn test_translation_mode_all_variants() {
        let modes = [
            TranslationMode::NewTranslation,
            TranslationMode::DiffTranslation,
            TranslationMode::OverwriteUpdate,
            TranslationMode::ManualEdit,
        ];

        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: TranslationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, mode);
        }
    }

    #[test]
    fn test_translation_engine_type_default() {
        let default: TranslationEngineType = Default::default();
        assert_eq!(default, TranslationEngineType::DeepL);
    }

    #[test]
    fn test_translation_engine_type_serialization() {
        let google = TranslationEngineType::GoogleTranslate;
        let json = serde_json::to_string(&google).unwrap();
        assert_eq!(json, "\"GoogleTranslate\"");

        let deserialized: TranslationEngineType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TranslationEngineType::GoogleTranslate);
    }

    #[test]
    fn test_translation_source_all_variants() {
        let sources = [
            TranslationSource::API,
            TranslationSource::VanillaKeyMatch,
            TranslationSource::VanillaTextMatch,
            TranslationSource::Manual,
            TranslationSource::History,
            TranslationSource::Glossary,
        ];

        for source in sources {
            let json = serde_json::to_string(&source).unwrap();
            let deserialized: TranslationSource = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, source);
        }
    }

    #[test]
    fn test_translation_mode_equality() {
        assert_eq!(
            TranslationMode::NewTranslation,
            TranslationMode::NewTranslation
        );
        assert_ne!(
            TranslationMode::NewTranslation,
            TranslationMode::DiffTranslation
        );
    }

    #[test]
    fn test_translation_source_equality() {
        assert_eq!(TranslationSource::API, TranslationSource::API);
        assert_ne!(TranslationSource::API, TranslationSource::VanillaKeyMatch);
    }
}
