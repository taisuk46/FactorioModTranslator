use regex::Regex;
use html_escape::{encode_safe, decode_html_entities};

pub struct FactorioStringFormatter;

impl FactorioStringFormatter {
    pub fn wrap_tags(text: &str) -> String {
        if text.is_empty() {
            return text.to_string();
        }

        // 1. HtmlEncode to make it safe for XML TagHandling in DeepL
        let encoded = encode_safe(text).to_string();

        // 2. Wrap protected parts in <keep> tags
        // Regex for: __...__, [...], and \n
        let re = Regex::new(r"((?:__.*?__)+)|(\[.*?\])|(\\n)").unwrap();
        
        re.replace_all(&encoded, |caps: &regex::Captures| {
            format!("<keep>{}</keep>", &caps[0])
        }).to_string()
    }

    pub fn unwrap_tags(translated_text: &str) -> String {
        if translated_text.is_empty() {
            return translated_text.to_string();
        }

        // 1. Remove <keep> tags
        let unwrapped = translated_text.replace("<keep>", "").replace("</keep>", "");

        // 2. HtmlDecode to restore original symbols correctly
        decode_html_entities(&unwrapped).to_string()
    }
}
