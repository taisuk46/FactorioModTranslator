use serde::{Deserialize, Serialize};
use crate::models::enums::ModSourceType;
use crate::models::cfg::CfgFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub version: String,
    pub title: String,
    pub author: String,
    pub source_path: String,
    pub source_type: ModSourceType,
    pub factorio_version: Option<String>,
    pub locale_files: Vec<CfgFile>,
}
