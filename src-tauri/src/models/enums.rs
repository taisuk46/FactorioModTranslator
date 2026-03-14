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
