use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::services::cfg_parser::CfgParser;

pub struct VanillaTranslationService {
    key_to_value: HashMap<String, String>,
    text_to_value: HashMap<String, String>,
}

impl VanillaTranslationService {
    pub fn new() -> Self {
        Self {
            key_to_value: HashMap::new(),
            text_to_value: HashMap::new(),
        }
    }

    /// Loads vanilla Factorio locale data from the installation path.
    pub fn load_vanilla_data(&mut self, factorio_path: &str, lang_code: &str) -> Result<(), String> {
        self.key_to_value.clear();
        self.text_to_value.clear();

        let data_path = Path::new(factorio_path).join("data");
        if !data_path.exists() {
            return Err(format!("Data path not found: {}", data_path.display()));
        }

        if let Ok(entries) = fs::read_dir(&data_path) {
            for entry in entries.flatten() {
                let module_path = entry.path();
                if module_path.is_dir() {
                    let locale_path = module_path.join("locale").join(lang_code);
                    if locale_path.exists() {
                        self.load_from_locale_path(&locale_path, true);
                    }
                }
            }
        }

        // To support text-based matching properly, we should also load English data
        self.load_english_data(factorio_path);

        Ok(())
    }

    fn load_english_data(&mut self, factorio_path: &str) {
        let data_path = Path::new(factorio_path).join("data");
        if let Ok(entries) = fs::read_dir(&data_path) {
            for entry in entries.flatten() {
                let module_path = entry.path();
                if module_path.is_dir() {
                    let locale_path = module_path.join("locale").join("en");
                    if locale_path.exists() {
                        self.load_from_locale_path(&locale_path, false);
                    }
                }
            }
        }
    }

    fn load_from_locale_path(&mut self, locale_path: &Path, is_target_lang: bool) {
        if let Ok(files) = fs::read_dir(locale_path) {
            for file_entry in files.flatten() {
                let cfg_path = file_entry.path();
                if cfg_path.extension().and_then(|s| s.to_str()) == Some("cfg") {
                    if let Ok(file) = fs::File::open(&cfg_path) {
                        let reader = std::io::BufReader::new(file);
                        let parsed = CfgParser::parse(reader, String::new(), String::new());
                        for entry in parsed.entries {
                            let full_key = format!("{}.{}", entry.section, entry.key);
                            if is_target_lang {
                                self.key_to_value.insert(full_key, entry.value);
                            } else {
                                // For English data, map English text -> Target translation if we have it by key
                                if let Some(target_value) = self.key_to_value.get(&full_key) {
                                    self.text_to_value.insert(entry.value, target_value.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn match_by_key(&self, section: &str, key: &str) -> Option<String> {
        let full_key = format!("{}.{}", section, key);
        self.key_to_value.get(&full_key).cloned()
    }

    pub fn match_by_text(&self, source_text: &str) -> Option<String> {
        self.text_to_value.get(source_text).cloned()
    }

    pub fn get_context_hints(&self, source_text: &str) -> Vec<String> {
        self.text_to_value.iter()
            .filter(|(k, _)| k.to_lowercase().contains(&source_text.to_lowercase()))
            .take(3)
            .map(|(k, v)| format!("{} → {}", k, v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_vanilla_data_matching_success_should_find_matches() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("data");
        let core_locale_ja = data_dir.join("core/locale/ja");
        let base_locale_ja = data_dir.join("base/locale/ja");
        let core_locale_en = data_dir.join("core/locale/en");
        let base_locale_en = data_dir.join("base/locale/en");
        
        fs::create_dir_all(&core_locale_ja).unwrap();
        fs::create_dir_all(&base_locale_ja).unwrap();
        fs::create_dir_all(&core_locale_en).unwrap();
        fs::create_dir_all(&base_locale_en).unwrap();

        fs::write(core_locale_ja.join("core.cfg"), "[gui]\nconfirm=確定").unwrap();
        fs::write(base_locale_ja.join("base.cfg"), "[item-name]\niron-plate=鉄板").unwrap();
        
        fs::write(core_locale_en.join("core.cfg"), "[gui]\nconfirm=Confirm").unwrap();
        fs::write(base_locale_en.join("base.cfg"), "[item-name]\niron-plate=Iron Plate").unwrap();

        let mut service = VanillaTranslationService::new();
        service.load_vanilla_data(&dir.path().to_string_lossy(), "ja").unwrap();

        // Key match
        assert_eq!(service.match_by_key("item-name", "iron-plate"), Some("鉄板".to_string()));
        
        // Text match
        assert_eq!(service.match_by_text("Iron Plate"), Some("鉄板".to_string()));
    }
}
