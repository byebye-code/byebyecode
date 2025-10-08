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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageData {
    #[serde(rename = "totalTokens")]
    pub total_tokens: u64,
    #[serde(rename = "creditLimit")]
    pub credit_limit: f64,
    #[serde(rename = "currentCredits")]
    pub current_credits: f64,

    // 计算字段(序列化时保存,从缓存读取时也能用)
    #[serde(default)]
    pub used_tokens: u64,
    #[serde(default)]
    pub remaining_tokens: u64,
    #[serde(default)]
    pub percentage_used: f64,
}

impl UsageData {
    /// 计算已使用和剩余的积分
    pub fn calculate(&mut self) {
        // 使用积分而不是token
        // creditLimit = 20.0刀, currentCredits = 15.35刀
        // 已使用积分 = 20 - 15.35 = 4.65刀
        // 使用百分比 = (20 - 15.35) / 20 * 100 = 23.25%

        let used_credits = self.credit_limit - self.current_credits;
        self.percentage_used = if self.credit_limit > 0.0 {
            (used_credits / self.credit_limit * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        // 直接使用积分(美元)进行显示
        // used_tokens 和 remaining_tokens 现在表示积分(以美分为单位,便于整数显示)
        self.used_tokens = (used_credits * 100.0) as u64; // 转换为美分
        self.remaining_tokens = (self.current_credits * 100.0) as u64;
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

/// Read API key from Claude settings.json if base URL is 88code.org
pub fn get_api_key_from_claude_settings() -> Option<String> {
    let settings_path = get_claude_settings_path()?;

    if !settings_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(settings_path).ok()?;
    let settings: ClaudeSettings = serde_json::from_str(&content).ok()?;

    let env = settings.env?;

    // Only use the API key if base URL is 88code.org
    if let Some(base_url) = env.base_url {
        if base_url.contains("88code.org") {
            return env.auth_token;
        }
    }

    None
}
