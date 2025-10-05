pub mod glm;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_url: String,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            api_url: "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string(),
        }
    }
}

pub trait Translator {
    fn translate_to_english(&self, chinese_text: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn translate_to_chinese(&self, english_text: &str) -> Result<String, Box<dyn std::error::Error>>;
}
