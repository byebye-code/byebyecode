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

/// 后端统一响应包装器
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseDTO<T> {
    pub code: i32,
    pub ok: bool,
    pub msg: String,
    pub data: T,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: String::new(),
            usage_url: "https://www.88code.ai/api/usage".to_string(),
            subscription_url: "https://www.88code.ai/api/subscription".to_string(),
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

    /// 订阅实体列表，包含所有套餐的详细信息
    #[serde(rename = "subscriptionEntityList", default)]
    pub subscription_entity_list: Vec<SubscriptionEntity>,

    #[serde(default)]
    pub used_tokens: u64,
    #[serde(default)]
    pub remaining_tokens: u64,
    #[serde(default)]
    pub percentage_used: f64,
}

/// 订阅实体（从 subscriptionEntityList 解析）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionEntity {
    #[serde(rename = "subscriptionName")]
    pub subscription_name: String,
    #[serde(rename = "creditLimit")]
    pub credit_limit: f64,
    #[serde(rename = "currentCredits")]
    pub current_credits: f64,
    #[serde(rename = "isActive")]
    pub is_active: bool,
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
        // 从 subscriptionEntityList 中找到正在扣费的套餐
        // Claude Code 环境下跳过 FREE 套餐（FREE 不支持 CC）
        // 选择第一个有消费（currentCredits < creditLimit）的非 FREE 活跃套餐
        let active_subscription = self
            .subscription_entity_list
            .iter()
            .filter(|s| s.is_active)
            .filter(|s| s.subscription_name.to_uppercase() != "FREE") // 跳过 FREE
            .find(|s| s.current_credits < s.credit_limit);

        // 如果找到正在扣费的套餐，用那个套餐的数据
        let (credit_limit, current_credits) = match active_subscription {
            Some(sub) => (sub.credit_limit, sub.current_credits),
            None => (self.credit_limit, self.current_credits),
        };

        let used_credits = credit_limit - current_credits;
        self.percentage_used = if credit_limit > 0.0 {
            (used_credits / credit_limit * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        self.used_tokens = (used_credits * 100.0).max(0.0) as u64;

        if current_credits < 0.0 {
            self.remaining_tokens = 0;
        } else {
            self.remaining_tokens = (current_credits * 100.0) as u64;
        }

        // 更新顶层数据为实际使用的套餐数据
        self.credit_limit = credit_limit;
        self.current_credits = current_credits;
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

/// Read API key from Claude settings.json if base URL is 88code or packyapi
pub fn get_api_key_from_claude_settings() -> Option<String> {
    let settings_path = get_claude_settings_path()?;

    if !settings_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(settings_path).ok()?;
    let settings: ClaudeSettings = serde_json::from_str(&content).ok()?;

    let env = settings.env?;

    // Support 88code (both .org and .ai), packyapi.com, and rainapp.top (国内线路)
    if let Some(base_url) = env.base_url {
        if base_url.contains("88code.org")
            || base_url.contains("88code.ai")
            || base_url.contains("packyapi.com")
            || base_url.contains("rainapp.top")
        {
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
    } else if base_url.contains("88code.ai") || base_url.contains("rainapp.top") {
        // 新域名：88code.ai 和国内线路 rainapp.top
        Some("https://www.88code.ai/api/usage".to_string())
    } else if base_url.contains("88code.org") {
        // 旧域名兼容
        Some("https://www.88code.org/api/usage".to_string())
    } else {
        None
    }
}
