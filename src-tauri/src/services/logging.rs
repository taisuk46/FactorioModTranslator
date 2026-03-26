use log::info;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

pub struct LogContext {
    pub request_id: String,
    pub function_name: String,
    pub start_time: Instant,
}

impl LogContext {
    pub fn new(function_name: &str) -> Self {
        let request_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();

        info!(
            "{}",
            json!({
                "event": "started",
                "request_id": request_id,
                "function": function_name,
            })
        );

        Self {
            request_id,
            function_name: function_name.to_string(),
            start_time,
        }
    }

    pub fn complete(&self) {
        let duration_ms = self.start_time.elapsed().as_millis();
        info!(
            "{}",
            json!({
                "event": "completed",
                "request_id": self.request_id,
                "function": self.function_name,
                "duration_ms": duration_ms,
            })
        );
    }

    pub fn error(&self, error: &str) {
        let duration_ms = self.start_time.elapsed().as_millis();
        log::error!(
            "{}",
            json!({
                "event": "failed",
                "request_id": self.request_id,
                "function": self.function_name,
                "duration_ms": duration_ms,
                "error": error,
            })
        );
    }
}

pub fn mask_sensitive(s: &str) -> String {
    if s.len() <= 8 {
        "***".to_string()
    } else {
        format!("{}***", &s[..4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_sensitive_short_string() {
        let result = mask_sensitive("short");
        assert_eq!(result, "***");
    }

    #[test]
    fn test_mask_sensitive_exactly_8_chars() {
        let result = mask_sensitive("12345678");
        assert_eq!(result, "***");
    }

    #[test]
    fn test_mask_sensitive_long_string() {
        let result = mask_sensitive("my-api-key-12345");
        assert_eq!(result, "my-a***");
    }

    #[test]
    fn test_mask_sensitive_empty_string() {
        let result = mask_sensitive("");
        assert_eq!(result, "***");
    }

    #[test]
    fn test_mask_sensitive_9_chars() {
        let result = mask_sensitive("123456789");
        assert_eq!(result, "1234***");
    }

    #[test]
    fn test_mask_sensitive_preserves_first_four() {
        let result = mask_sensitive("DeepL-key-fx");
        assert_eq!(result, "Deep***");
    }
}
