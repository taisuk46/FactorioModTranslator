use serde::{Deserialize, Serialize};
use crate::models::enums::TranslationEngineType;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub selected_engine: TranslationEngineType,
    pub factorio_install_path: String,
    pub ui_language: String,
    pub last_mod_path: String,
    pub window_width: u32,
    pub window_height: u32,
}
