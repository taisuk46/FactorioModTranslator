use async_trait::async_trait;
use serde::Deserialize;
use reqwest::Client;
use crate::services::formatter::FactorioStringFormatter;

#[async_trait]
pub trait TranslationEngine: Send + Sync {
    fn name(&self) -> &str;
    async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String, String>;
    async fn translate_batch(&self, texts: Vec<String>, source_lang: &str, target_lang: &str) -> Result<Vec<String>, String>;
}

pub struct DeepLTranslationEngine {
    api_key: String,
    client: Client,
}

impl DeepLTranslationEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn map_lang(lang: &str, is_target: bool) -> String {
        let lang = lang.to_uppercase();
        if lang == "EN" {
            if is_target { "EN-US".to_string() } else { "EN".to_string() }
        } else {
            lang
        }
    }
}

#[derive(Deserialize)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Deserialize)]
struct DeepLTranslation {
    text: String,
}

#[async_trait]
impl TranslationEngine for DeepLTranslationEngine {
    fn name(&self) -> &str { "DeepL" }

    async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String, String> {
        let wrapped_text = FactorioStringFormatter::wrap_tags(text);
        let url = if self.api_key.ends_with(":fx") {
            "https://api-free.deepl.com/v2/translate"
        } else {
            "https://api.deepl.com/v2/translate"
        };

        let params = [
            ("auth_key", &self.api_key),
            ("text", &wrapped_text),
            ("source_lang", &Self::map_lang(source_lang, false)),
            ("target_lang", &Self::map_lang(target_lang, true)),
            ("tag_handling", &"xml".to_string()),
            ("ignore_tags", &"keep".to_string()),
        ];

        let res = self.client.post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("DeepL API error: {}", res.status()));
        }

        let json: DeepLResponse = res.json().await.map_err(|e| e.to_string())?;
        let translated = json.translations.first().ok_or("No translations returned")?.text.clone();
        
        Ok(FactorioStringFormatter::unwrap_tags(&translated))
    }

    async fn translate_batch(&self, texts: Vec<String>, source_lang: &str, target_lang: &str) -> Result<Vec<String>, String> {
        let mut results = Vec::new();
        // Simple implementation: translate one by one for now (DeepL supports batching but form data needs care)
        for text in texts {
            results.push(self.translate(&text, source_lang, target_lang).await?);
        }
        Ok(results)
    }
}

pub struct GoogleTranslationEngine {
    api_key: String,
    client: Client,
}

impl GoogleTranslationEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct GoogleResponse {
    data: GoogleData,
}

#[derive(Deserialize)]
struct GoogleData {
    translations: Vec<GoogleTranslation>,
}

#[derive(Deserialize)]
struct GoogleTranslation {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[async_trait]
impl TranslationEngine for GoogleTranslationEngine {
    fn name(&self) -> &str { "Google Translate" }

    async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String, String> {
        let url = "https://translation.googleapis.com/language/translate/v2";
        
        let params = [
            ("key", &self.api_key),
            ("q", &text.to_string()),
            ("source", &source_lang.to_lowercase()),
            ("target", &target_lang.to_lowercase()),
            ("format", &"text".to_string()),
        ];

        let res = self.client.post(url)
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Google API error: {}", res.status()));
        }

        let json: GoogleResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(json.data.translations.first().ok_or("No translations returned")?.translated_text.clone())
    }

    async fn translate_batch(&self, texts: Vec<String>, source_lang: &str, target_lang: &str) -> Result<Vec<String>, String> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.translate(&text, source_lang, target_lang).await?);
        }
        Ok(results)
    }
}
