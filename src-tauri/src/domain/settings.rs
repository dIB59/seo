use serde::{Deserialize, Serialize};

/// Global application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub default_ai_provider: String,
    pub default_max_pages: i64,
    pub default_max_depth: i64,
    pub default_rate_limit_ms: i64,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            anthropic_api_key: None,
            google_api_key: None,
            default_ai_provider: "openai".to_string(),
            default_max_pages: 100,
            default_max_depth: 3,
            default_rate_limit_ms: 1000,
            theme: "system".to_string(),
        }
    }
}
