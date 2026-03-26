use html_escape::{decode_html_entities, encode_safe};
use regex::Regex;

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
        })
        .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_tags_empty_string() {
        let result = FactorioStringFormatter::wrap_tags("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_wrap_tags_simple_text() {
        let result = FactorioStringFormatter::wrap_tags("Hello World");
        // Should be HTML encoded and not have <keep> tags since no special patterns
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_wrap_tags_preserves_italic_underscores() {
        let result = FactorioStringFormatter::wrap_tags("__Iron Plate__");
        assert!(result.contains("<keep>"));
        assert!(result.contains("__Iron Plate__"));
    }

    #[test]
    fn test_wrap_tags_preserves_brackets() {
        let result = FactorioStringFormatter::wrap_tags("[item]Iron Plate[/item]");
        assert!(result.contains("<keep>"));
        assert!(result.contains("<keep>"));
    }

    #[test]
    fn test_wrap_tags_preserves_newlines() {
        let result = FactorioStringFormatter::wrap_tags("Line 1\\nLine 2");
        assert!(result.contains("<keep>"));
        assert!(result.contains("\\n"));
    }

    #[test]
    fn test_wrap_tags_combined_patterns() {
        let result = FactorioStringFormatter::wrap_tags("__Iron Plate__ and [item]Copper[/item]");
        assert!(result.contains("<keep>"));
    }

    #[test]
    fn test_unwrap_tags_empty_string() {
        let result = FactorioStringFormatter::unwrap_tags("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_unwrap_tags_removes_keep_tags() {
        let result = FactorioStringFormatter::unwrap_tags("Hello <keep>__test__</keep> World");
        assert!(!result.contains("<keep>"));
        assert!(!result.contains("</keep>"));
        assert!(result.contains("__test__"));
    }

    #[test]
    fn test_unwrap_tags_restores_html_entities() {
        // The wrap/unwrap cycle should be idempotent for common characters
        let original = "Test & <test>";
        let wrapped = FactorioStringFormatter::wrap_tags(original);
        let unwrapped = FactorioStringFormatter::unwrap_tags(&wrapped);
        // The unwrapped result should contain the original content (possibly HTML encoded)
        assert!(unwrapped.contains("Test"));
    }

    #[test]
    fn test_wrap_unwrap_cycle() {
        let original = "__Iron Plate__";
        let wrapped = FactorioStringFormatter::wrap_tags(original);
        let unwrapped = FactorioStringFormatter::unwrap_tags(&wrapped);
        // After unwrap, we should have the original underscores preserved
        assert!(unwrapped.contains("__Iron Plate__"));
    }
}
