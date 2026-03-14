use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgEntry {
    pub section: String,
    pub key: String,
    pub value: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgFile {
    pub file_path: String,
    pub language_code: String,
    pub entries: Vec<CfgEntry>,
    pub section_order: Vec<String>,
    pub header_comments: Vec<String>,
}
