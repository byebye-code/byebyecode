pub mod cache;
pub mod client;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub api_key: String,
    pub usage_url: String,
    pub subscription_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            usage_url: "https://www.88code.org/api/usage".to_string(),
            subscription_url: "https://www.88code.org/api/subscription".to_string(),
        }
    }
}

impl ApiConfig {
    pub fn is_packyapi(&self) -> bool {
        self.usage_url.contains("packyapi.com")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum UsageData {
    Code88(Code88UsageData),
    Packy(PackyUsageData),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Code88UsageData {
    #[serde(rename = "totalTokens")]
    pub total_tokens: u64,
    #[serde(rename = "creditLimit")]
    pub credit_limit: f64,
    #[serde(rename = "currentCredits")]
    pub current_credits: f64,

    #[serde(default)]
    pub used_tokens: u64,
    #[serde(default)]
    pub remaining_tokens: u64,
    #[serde(default)]
    pub percentage_used: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackyUsageResponse {
    pub code: bool,
    pub data: PackyUsageData,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackyUsageData {
    pub expires_at: i64,
    pub name: String,
    pub object: String,
    pub total_available: u64,
    pub total_granted: u64,
    pub total_used: u64,
    pub unlimited_quota: bool,

    #[serde(default)]
    pub used_tokens: u64,
    #[serde(default)]
    pub remaining_tokens: u64,
    #[serde(default)]
    pub percentage_used: f64,
    #[serde(default)]
    pub credit_limit: f64,
    #[serde(default)]
    pub current_credits: f64,
}

impl UsageData {
    pub fn calculate(&mut self) {
        match self {
            UsageData::Code88(data) => data.calculate(),
            UsageData::Packy(data) => data.calculate(),
        }
    }

    pub fn is_exhausted(&self) -> bool {
        match self {
            UsageData::Code88(data) => data.is_exhausted(),
            UsageData::Packy(data) => data.is_exhausted(),
        }
    }

    pub fn get_used_tokens(&self) -> u64 {
        match self {
            UsageData::Code88(data) => data.used_tokens,
            UsageData::Packy(data) => data.used_tokens,
        }
    }

    pub fn get_remaining_tokens(&self) -> u64 {
        match self {
            UsageData::Code88(data) => data.remaining_tokens,
            UsageData::Packy(data) => data.remaining_tokens,
        }
    }

    pub fn get_credit_limit(&self) -> f64 {
        match self {
            UsageData::Code88(data) => data.credit_limit,
            UsageData::Packy(data) => data.credit_limit,
        }
    }
}

impl Code88UsageData {
    pub fn calculate(&mut self) {
        let used_credits = self.credit_limit - self.current_credits;
        self.percentage_used = if self.credit_limit > 0.0 {
            (used_credits / self.credit_limit * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        self.used_tokens = (used_credits * 100.0).max(0.0) as u64;

        if self.current_credits < 0.0 {
            self.remaining_tokens = 0;
        } else {
            self.remaining_tokens = (self.current_credits * 100.0) as u64;
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.current_credits <= 0.0
    }
}

impl PackyUsageData {
    pub fn calculate(&mut self) {
        self.used_tokens = self.total_used;
        self.remaining_tokens = self.total_available.saturating_sub(self.total_used);

        self.percentage_used = if self.total_granted > 0 {
            (self.total_used as f64 / self.total_granted as f64 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        self.credit_limit = (self.total_granted as f64) / 100.0;
        self.current_credits = (self.remaining_tokens as f64) / 100.0;
    }

    pub fn is_exhausted(&self) -> bool {
        !self.unlimited_quota && self.remaining_tokens == 0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionData {
    #[serde(rename = "subscriptionPlanName")]
    pub plan_name: String,
    pub cost: f64,
    #[serde(rename = "endDate")]
    pub expires_at: Option<String>,
    #[serde(rename = "subscriptionStatus")]
    pub status: String,
    #[serde(rename = "remainingDays")]
    pub remaining_days: i32,
    #[serde(rename = "billingCycleDesc")]
    pub billing_cycle_desc: String,
    #[serde(rename = "resetTimes")]
    pub reset_times: i32,
    #[serde(rename = "isActive")]
    pub is_active: bool,

    // 计算字段
    #[serde(skip)]
    pub plan_price: String,
}

impl SubscriptionData {
    /// 格式化显示数据
    pub fn format(&mut self) {
        self.plan_price = format!("¥{}/{}", self.cost, self.billing_cycle_desc);
    }
}

/// Claude settings.json structure for reading API key
#[derive(Debug, Deserialize)]
struct ClaudeSettings {
    env: Option<ClaudeEnv>,
}

#[derive(Debug, Deserialize)]
struct ClaudeEnv {
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    auth_token: Option<String>,
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    base_url: Option<String>,
}

/// Get the path to Claude settings.json (cross-platform)
fn get_claude_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".claude").join("settings.json"))
}

/// Read API key from Claude settings.json if base URL is 88code.org or packyapi.com
pub fn get_api_key_from_claude_settings() -> Option<String> {
    let settings_path = get_claude_settings_path()?;

    if !settings_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(settings_path).ok()?;
    let settings: ClaudeSettings = serde_json::from_str(&content).ok()?;

    let env = settings.env?;

    // Support both 88code.org and packyapi.com
    if let Some(base_url) = env.base_url {
        if base_url.contains("88code.org") || base_url.contains("packyapi.com") {
            return env.auth_token;
        }
    }

    None
}

/// Get usage_url from Claude settings.json based on ANTHROPIC_BASE_URL
pub fn get_usage_url_from_claude_settings() -> Option<String> {
    let settings_path = get_claude_settings_path()?;

    if !settings_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(settings_path).ok()?;
    let settings = serde_json::from_str::<serde_json::Value>(&content).ok()?;

    let base_url = settings.get("env")?.get("ANTHROPIC_BASE_URL")?.as_str()?;

    if base_url.contains("packyapi.com") {
        Some("https://www.packyapi.com/api/usage/token/".to_string())
    } else if base_url.contains("88code.org") {
        Some("https://www.88code.org/api/usage".to_string())
    } else {
        None
    }
}
