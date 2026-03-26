use crate::models::cfg::CfgFile;
use crate::models::enums::ModSourceType;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::cfg::CfgEntry;

    #[test]
    fn test_mod_info_creation() {
        let mod_info = ModInfo {
            name: "my-mod".to_string(),
            version: "1.0.0".to_string(),
            title: "My Mod".to_string(),
            author: "Author".to_string(),
            source_path: "/path/to/mod".to_string(),
            source_type: ModSourceType::Folder,
            factorio_version: Some("2.0".to_string()),
            locale_files: vec![],
        };

        assert_eq!(mod_info.name, "my-mod");
        assert_eq!(mod_info.version, "1.0.0");
        assert_eq!(mod_info.source_type, ModSourceType::Folder);
    }

    #[test]
    fn test_mod_info_zip_source_type() {
        let mod_info = ModInfo {
            name: "my-mod".to_string(),
            version: "0.5.0".to_string(),
            title: "My Mod".to_string(),
            author: "Author".to_string(),
            source_path: "/path/to/mod.zip".to_string(),
            source_type: ModSourceType::Zip,
            factorio_version: None,
            locale_files: vec![],
        };

        assert_eq!(mod_info.source_type, ModSourceType::Zip);
        assert!(mod_info.factorio_version.is_none());
    }

    #[test]
    fn test_mod_info_with_locale_files() {
        let cfg_file = CfgFile {
            file_path: "locale/en/strings.cfg".to_string(),
            language_code: "en".to_string(),
            entries: vec![CfgEntry {
                section: "item-name".to_string(),
                key: "iron-plate".to_string(),
                value: "Iron Plate".to_string(),
                comment: None,
            }],
            section_order: vec!["item-name".to_string()],
            header_comments: vec![],
        };

        let mod_info = ModInfo {
            name: "my-mod".to_string(),
            version: "1.0.0".to_string(),
            title: "My Mod".to_string(),
            author: "Author".to_string(),
            source_path: "/path/to/mod".to_string(),
            source_type: ModSourceType::Folder,
            factorio_version: Some("2.0".to_string()),
            locale_files: vec![cfg_file],
        };

        assert_eq!(mod_info.locale_files.len(), 1);
        assert_eq!(mod_info.locale_files[0].entries.len(), 1);
    }

    #[test]
    fn test_mod_info_serialization() {
        let mod_info = ModInfo {
            name: "test-mod".to_string(),
            version: "2.0.0".to_string(),
            title: "Test Mod".to_string(),
            author: "Test Author".to_string(),
            source_path: "/test/path".to_string(),
            source_type: ModSourceType::Folder,
            factorio_version: Some("1.1".to_string()),
            locale_files: vec![],
        };

        let json = serde_json::to_string(&mod_info).unwrap();
        assert!(json.contains("test-mod"));
        assert!(json.contains("2.0.0"));

        let deserialized: ModInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-mod");
        assert_eq!(deserialized.version, "2.0.0");
    }
}
