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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_entry_creation() {
        let entry = CfgEntry {
            section: "item-name".to_string(),
            key: "iron-plate".to_string(),
            value: "Iron Plate".to_string(),
            comment: Some("Comment".to_string()),
        };

        assert_eq!(entry.section, "item-name");
        assert_eq!(entry.key, "iron-plate");
        assert_eq!(entry.value, "Iron Plate");
        assert_eq!(entry.comment, Some("Comment".to_string()));
    }

    #[test]
    fn test_cfg_entry_without_comment() {
        let entry = CfgEntry {
            section: "item-name".to_string(),
            key: "copper-plate".to_string(),
            value: "Copper Plate".to_string(),
            comment: None,
        };

        assert!(entry.comment.is_none());
    }

    #[test]
    fn test_cfg_entry_serialization() {
        let entry = CfgEntry {
            section: "item-name".to_string(),
            key: "iron-plate".to_string(),
            value: "Iron Plate".to_string(),
            comment: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("item-name"));
        assert!(json.contains("iron-plate"));
        assert!(json.contains("Iron Plate"));
    }

    #[test]
    fn test_cfg_file_creation() {
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
            header_comments: vec!["; Header comment".to_string()],
        };

        assert_eq!(cfg_file.file_path, "locale/en/strings.cfg");
        assert_eq!(cfg_file.language_code, "en");
        assert_eq!(cfg_file.entries.len(), 1);
    }

    #[test]
    fn test_cfg_file_empty_entries() {
        let cfg_file = CfgFile {
            file_path: "locale/en/strings.cfg".to_string(),
            language_code: "en".to_string(),
            entries: vec![],
            section_order: vec![],
            header_comments: vec![],
        };

        assert!(cfg_file.entries.is_empty());
        assert!(cfg_file.section_order.is_empty());
        assert!(cfg_file.header_comments.is_empty());
    }
}
