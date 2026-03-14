use std::collections::HashMap;

pub struct LocalizationService {
    translations: HashMap<String, HashMap<String, String>>,
}

impl LocalizationService {
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        
        // Japanese
        let mut ja = HashMap::new();
        ja.insert("AppTitle".to_string(), "Factorio Mod 自動翻訳ツール".to_string());
        ja.insert("SelectMod".to_string(), "Modを選択".to_string());
        ja.insert("Translate".to_string(), "翻訳".to_string());
        ja.insert("Glossary".to_string(), "用語集".to_string());
        ja.insert("History".to_string(), "履歴".to_string());
        ja.insert("Settings".to_string(), "設定".to_string());
        ja.insert("PromptEnterModPath".to_string(), "ModフォルダまたはZIPのパスを入力してください:".to_string());
        ja.insert("TitleModSelection".to_string(), "Modプロジェクトを選択".to_string());
        ja.insert("LabelDropMod".to_string(), "ここにModをドロップするか、参照ボタンで選択してください".to_string());
        ja.insert("BtnBrowse".to_string(), "ファイルを参照...".to_string());
        ja.insert("BtnTranslate".to_string(), "すべて翻訳実行".to_string());
        ja.insert("BtnSaveMod".to_string(), "翻訳済みModとして保存".to_string());
        ja.insert("LabelEngine".to_string(), "翻訳エンジン".to_string());
        ja.insert("LabelApiKey".to_string(), "APIキー (安全に保存されます)".to_string());
        ja.insert("BtnSaveKey".to_string(), "設定を適用".to_string());
        translations.insert("ja".to_string(), ja);
        
        // English
        let mut en = HashMap::new();
        en.insert("AppTitle".to_string(), "Factorio Mod Auto-Translator".to_string());
        en.insert("SelectMod".to_string(), "Select Mod".to_string());
        en.insert("Translate".to_string(), "Translate".to_string());
        en.insert("Glossary".to_string(), "Glossary".to_string());
        en.insert("History".to_string(), "History".to_string());
        en.insert("Settings".to_string(), "Settings".to_string());
        en.insert("PromptEnterModPath".to_string(), "Enter Mod Folder or Zip path:".to_string());
        en.insert("TitleModSelection".to_string(), "Select Mod Project".to_string());
        en.insert("LabelDropMod".to_string(), "Drag & Drop Mod Folder or ZIP here".to_string());
        en.insert("BtnBrowse".to_string(), "Browse Files...".to_string());
        en.insert("BtnTranslate".to_string(), "Translate All".to_string());
        en.insert("BtnSaveMod".to_string(), "Save Translated Mod".to_string());
        en.insert("LabelEngine".to_string(), "Translation Engine".to_string());
        en.insert("LabelApiKey".to_string(), "API Key (Securely stored)".to_string());
        en.insert("BtnSaveKey".to_string(), "Apply Changes".to_string());
        translations.insert("en".to_string(), en);
        
        Self { translations }
    }

    pub fn get_string(&self, key: &str, lang: &str) -> String {
        let lang_code = if lang.starts_with("ja") { "ja" } else { "en" };
        
        self.translations.get(lang_code)
            .and_then(|m| m.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn get_all_translations(&self, lang: &str) -> HashMap<String, String> {
        let lang_code = if lang.starts_with("ja") { "ja" } else { "en" };
        self.translations.get(lang_code).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localization_should_return_correct_strings() {
        let service = LocalizationService::new();
        
        assert_eq!(service.get_string("AppTitle", "ja"), "Factorio Mod 自動翻訳ツール");
        assert_eq!(service.get_string("AppTitle", "en"), "Factorio Mod Auto-Translator");
        assert_eq!(service.get_string("InvalidKey", "ja"), "InvalidKey");
    }

    #[test]
    fn test_get_all_translations_should_return_entire_map() {
        let service = LocalizationService::new();
        let ja = service.get_all_translations("ja");
        assert!(ja.contains_key("AppTitle"));
        assert_eq!(ja.get("AppTitle").unwrap(), "Factorio Mod 自動翻訳ツール");
    }
}
