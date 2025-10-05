use super::{Translator, TranslationConfig};
use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct GLMRequest {
    model: String,
    messages: Vec<GLMMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GLMMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GLMResponse {
    choices: Vec<GLMChoice>,
}

#[derive(Debug, Deserialize)]
struct GLMChoice {
    message: GLMMessage,
}

#[derive(Clone)]
pub struct GLMTranslator {
    config: TranslationConfig,
    client: Arc<Client>,
}

impl GLMTranslator {
    pub fn new(config: TranslationConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Arc::new(Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?);

        Ok(Self { config, client })
    }

    fn call_api(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = GLMRequest {
            model: "glm-4-flash".to_string(),
            messages: vec![GLMMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self.client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("API request failed: {}", response.status()).into());
        }

        let glm_response: GLMResponse = response.json()?;

        if let Some(choice) = glm_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from GLM API".into())
        }
    }
}

impl Translator for GLMTranslator {
    fn translate_to_english(&self, chinese_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "请将以下中文翻译成信达雅的英文,只返回翻译结果,不要有任何解释:\n\n{}",
            chinese_text
        );
        self.call_api(&prompt)
    }

    fn translate_to_chinese(&self, english_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "请将以下英文翻译成流畅的中文,只返回翻译结果,不要有任何解释:\n\n{}",
            english_text
        );
        self.call_api(&prompt)
    }
}
