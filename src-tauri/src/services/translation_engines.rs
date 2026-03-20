use async_trait::async_trait;
use serde::Deserialize;
use reqwest::Client;
use crate::services::formatter::FactorioStringFormatter;
use crate::services::logging::mask_sensitive;
use log::{info, error, warn};
use serde_json::json;

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
        let client = Client::builder()
            .user_agent("FactorioModTranslator/0.1.0")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            api_key,
            client,
        }
    }

    fn map_lang(lang: &str, is_target: bool) -> String {
        let lang = lang.to_uppercase();
        if lang == "EN" {
            if is_target { "EN-US".to_string() } else { "EN".to_string() }
        } else if lang == "ZH-CN" {
            "ZH".to_string()
        } else if lang == "PT-BR" || lang == "PT-PT" {
            if is_target { lang } else { "PT".to_string() }
        } else if lang.starts_with("ES-") {
            "ES".to_string()
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
        let trimmed_key = self.api_key.trim();
        let url = if trimmed_key.ends_with(":fx") {
            "https://api-free.deepl.com/v2/translate"
        } else {
            "https://api.deepl.com/v2/translate"
        };

        let params = [
            ("text", &wrapped_text),
            ("source_lang", &Self::map_lang(source_lang, false)),
            ("target_lang", &Self::map_lang(target_lang, true)),
            ("tag_handling", &"xml".to_string()),
            ("ignore_tags", &"keep".to_string()),
        ];

        let mut retry_count = 0;
        let max_retries = 5;
        let mut delay = std::time::Duration::from_millis(1000);

        loop {
            info!("{}", json!({
                "event": "api_request",
                "engine": "DeepL",
                "attempt": retry_count + 1,
                "url": url,
                "source": source_lang,
                "target": target_lang,
                "text_len": text.len(),
                "api_key_masked": mask_sensitive(&self.api_key),
            }));

            let res_result = self.client.post(url)
                .header("Authorization", format!("DeepL-Auth-Key {}", trimmed_key))
                .form(&params)
                .send()
                .await;

            match res_result {
                Ok(res) => {
                    let status = res.status();
                    if status.is_success() {
                        let json: DeepLResponse = res.json().await.map_err(|e| {
                            error!("{}", json!({ "event": "api_parse_error", "engine": "DeepL", "error": e.to_string() }));
                            e.to_string()
                        })?;
                        
                        let translated = json.translations.first().ok_or("No translations returned")?.text.clone();
                        return Ok(FactorioStringFormatter::unwrap_tags(&translated));
                    } else if (status.as_u16() == 429 || status.is_server_error()) && retry_count < max_retries {
                        warn!("{}", json!({
                            "event": "api_retryable_error",
                            "engine": "DeepL",
                            "status": status.as_u16(),
                            "retry_count": retry_count + 1,
                            "next_delay_ms": delay.as_millis()
                        }));
                    } else {
                        error!("{}", json!({ "event": "api_error_status", "engine": "DeepL", "status": status.as_u16() }));
                        return Err(format!("DeepL API error: {}", status));
                    }
                }
                Err(e) if retry_count < max_retries => {
                    warn!("{}", json!({
                        "event": "api_network_retry",
                        "engine": "DeepL",
                        "error": e.to_string(),
                        "retry_count": retry_count + 1,
                        "next_delay_ms": delay.as_millis()
                    }));
                }
                Err(e) => {
                    error!("{}", json!({ "event": "api_network_error", "engine": "DeepL", "error": e.to_string() }));
                    return Err(e.to_string());
                }
            }

            tokio::time::sleep(delay).await;
            retry_count += 1;
            delay *= 2;
        }
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
        
        info!("{}", json!({
            "event": "api_request",
            "engine": "Google",
            "source": source_lang,
            "target": target_lang,
            "text_len": text.len(),
            "api_key_masked": mask_sensitive(&self.api_key),
        }));

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
            .map_err(|e| {
                error!("{}", json!({ "event": "api_network_error", "engine": "Google", "error": e.to_string() }));
                e.to_string()
            })?;

        let status = res.status();
        if !status.is_success() {
            error!("{}", json!({ "event": "api_error_status", "engine": "Google", "status": status.as_u16() }));
            return Err(format!("Google API error: {}", status));
        }

        let json: GoogleResponse = res.json().await.map_err(|e| {
            error!("{}", json!({ "event": "api_parse_error", "engine": "Google", "error": e.to_string() }));
            e.to_string()
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepl_endpoint_selection() {
        // Free API Key
        let free_engine = DeepLTranslationEngine::new("test-key:fx".to_string());
        let free_key = free_engine.api_key.trim();
        assert!(free_key.ends_with(":fx"));
        
        // Pro API Key
        let pro_engine = DeepLTranslationEngine::new("test-key-pro".to_string());
        let pro_key = pro_engine.api_key.trim();
        assert!(!pro_key.ends_with(":fx"));

        // Key with whitespace
        let ws_engine = DeepLTranslationEngine::new("  test-key:fx  \n".to_string());
        let ws_key = ws_engine.api_key.trim();
        assert!(ws_key.ends_with(":fx"));
        assert_eq!(ws_key, "test-key:fx");
    }
}
