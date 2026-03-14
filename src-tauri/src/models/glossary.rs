use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub source_term: String,
    pub target_term: String,
    pub source_lang: String,
    pub target_lang: String,
    pub exclude_from_translation: bool,
}
