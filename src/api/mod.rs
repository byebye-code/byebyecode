pub mod cache;
pub mod client;

use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

/// 自定义反序列化：将 null 转换为默认值 0.0
fn deserialize_null_as_zero<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<f64>::deserialize(deserializer)?;
    Ok(opt.unwrap_or(0.0))
}

/// 自定义反序列化：将 null 转换为空 Vec
fn deserialize_null_as_empty_vec<'de, D>(
    deserializer: D,
) -> Result<Vec<SubscriptionEntity>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Vec<SubscriptionEntity>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

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
    /// 判断是否是 88code 系列中转站
    /// 88code 使用特定的 API 格式（POST + ResponseDTO 包装）
    pub fn is_88code(&self) -> bool {
        self.usage_url.contains("88code.org")
            || self.usage_url.contains("88code.ai")
            || self.usage_url.contains("rainapp.top")
    }

    /// 判断是否是 Packy 中转站
    pub fn is_packy(&self) -> bool {
        self.usage_url.contains("packyapi.com")
    }

    /// 获取服务名称（用于状态栏显示）
    pub fn get_service_name(&self) -> &'static str {
        if self.is_88code() {
            "88code"
        } else if self.is_packy() {
            "packy"
        } else {
            "relay" // 其他中转站统一显示为 relay
        }
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
    #[serde(rename = "totalTokens", default)]
    pub total_tokens: u64,
    #[serde(
        rename = "creditLimit",
        default,
        deserialize_with = "deserialize_null_as_zero"
    )]
    pub credit_limit: f64,
    #[serde(
        rename = "currentCredits",
        default,
        deserialize_with = "deserialize_null_as_zero"
    )]
    pub current_credits: f64,

    /// 订阅实体列表，包含所有套餐的详细信息
    #[serde(
        rename = "subscriptionEntityList",
        default,
        deserialize_with = "deserialize_null_as_empty_vec"
    )]
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
    pub total_available: i64, // 改为 i64，支持负数（超额使用）
    pub total_granted: i64,
    pub total_used: i64,
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

    /// 判断是否只有 FREE 套餐（仅 88code 支持）
    pub fn has_only_free(&self) -> bool {
        match self {
            UsageData::Code88(data) => data.has_only_free(),
            UsageData::Packy(_) => false, // Packy 不支持
        }
    }

    /// 判断 usage 数据是否有效（用于检测 API 是否返回了有效数据）
    /// 如果无效，需要 fallback 到 subscription API
    pub fn is_valid(&self) -> bool {
        match self {
            UsageData::Code88(data) => data.is_valid(),
            UsageData::Packy(_) => true, // Packy 格式不受影响
        }
    }
}

impl Code88UsageData {
    /// 判断 usage API 返回的数据是否有效
    /// 当 API 返回 creditLimit=null, subscriptionEntityList=null 时为无效
    pub fn is_valid(&self) -> bool {
        // 有效条件：creditLimit > 0 或 subscriptionEntityList 非空
        self.credit_limit > 0.0 || !self.subscription_entity_list.is_empty()
    }

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

    /// 判断是否只有 FREE 套餐（没有 PLUS/PRO/MAX）
    /// 用于判断是否应该显示 PAYGO
    pub fn has_only_free(&self) -> bool {
        // 检查 subscriptionEntityList 中是否有 PLUS/PRO/MAX 套餐
        // 排除 FREE 和 PAYGO
        !self.subscription_entity_list.iter().any(|s| {
            if !s.is_active {
                return false;
            }
            let name = s.subscription_name.to_uppercase();
            name != "FREE" && name != "PAYGO"
        })
    }
}

impl PackyUsageData {
    pub fn calculate(&mut self) {
        // Packy API 返回的字段含义：
        // - total_granted: 套餐总额度（积分）
        // - total_used: 已使用额度（积分）
        // - total_available: 剩余可用额度（积分），可能为负数（超额使用）
        //
        // 单位转换：Packy 使用 500000 积分 = 1 美元（从用户实际数据推算）

        const PACKY_CONVERSION_FACTOR: f64 = 500000.0; // 500000 积分 = 1 美元

        // 计算美元金额
        let used_dollars = self.total_used as f64 / PACKY_CONVERSION_FACTOR;
        let remaining_dollars = self.total_available as f64 / PACKY_CONVERSION_FACTOR;
        let total_dollars = self.total_granted as f64 / PACKY_CONVERSION_FACTOR;

        // 转换为 cents（与 88code 统一，因为显示层会除以 100）
        // 处理负数：used_tokens 和 remaining_tokens 是 u64，需要 clamp 到 0
        self.used_tokens = (used_dollars * 100.0).max(0.0) as u64;
        self.remaining_tokens = (remaining_dollars * 100.0).max(0.0) as u64;

        // 百分比基于 total_granted（套餐总额度）计算
        // 超额使用时百分比可能超过 100%
        self.percentage_used = if self.total_granted > 0 {
            (self.total_used as f64 / self.total_granted as f64 * 100.0).max(0.0)
        } else {
            0.0
        };

        // 设置美元金额（用于 get_credit_limit 等方法）
        self.credit_limit = total_dollars.max(0.0);
        self.current_credits = remaining_dollars; // 可以是负数，表示超额
    }

    pub fn is_exhausted(&self) -> bool {
        !self.unlimited_quota && self.remaining_tokens == 0
    }
}

/// 套餐计划详情（嵌套在 SubscriptionData 中）
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SubscriptionPlan {
    #[serde(rename = "creditLimit", default)]
    pub credit_limit: f64,
    #[serde(rename = "subscriptionName", default)]
    pub subscription_name: String,
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
    /// 当前剩余额度（美元）- 用于 PAYGO 等套餐显示
    #[serde(rename = "currentCredits", default)]
    pub current_credits: f64,
    /// 套餐总额度（美元）- 顶层可能没有，需要从 subscription_plan 获取
    #[serde(rename = "creditLimit", default)]
    pub credit_limit: f64,
    /// 套餐计划详情 - 包含 creditLimit 等信息
    #[serde(rename = "subscriptionPlan", default)]
    pub subscription_plan: SubscriptionPlan,
    /// 订阅 ID - 用于排序（越小越早购买）
    #[serde(default)]
    pub id: i64,

    // 计算字段
    #[serde(skip)]
    pub plan_price: String,
}

impl SubscriptionData {
    /// 格式化显示数据，并从 subscription_plan 补充 credit_limit
    pub fn format(&mut self) {
        self.plan_price = format!("¥{}/{}", self.cost, self.billing_cycle_desc);
        // 如果顶层 credit_limit 为 0，从 subscription_plan 获取
        if self.credit_limit == 0.0 && self.subscription_plan.credit_limit > 0.0 {
            self.credit_limit = self.subscription_plan.credit_limit;
        }
    }

    /// 获取扣费优先级（用于排序）
    /// PLUS/PRO/MAX = 1, PAYGO = 2, FREE = 3
    fn billing_priority(&self) -> u8 {
        match self.plan_name.to_uppercase().as_str() {
            "FREE" => 3,
            "PAYGO" => 2,
            _ => 1, // PLUS, PRO, MAX 等
        }
    }
}

impl Code88UsageData {
    /// 从 subscription 数据构造 UsageData（fallback 方案）
    /// 当 /api/usage 返回无效数据时使用
    pub fn from_subscriptions(subscriptions: &[SubscriptionData]) -> Self {
        // 筛选活跃套餐：is_active && status == "活跃中"
        let mut active_subs: Vec<&SubscriptionData> = subscriptions
            .iter()
            .filter(|s| s.is_active)
            .filter(|s| s.status == "活跃中")
            .collect();

        // 按扣费优先级排序：PLUS/PRO/MAX > PAYGO > FREE
        // 同优先级按 id 排序（越小越早购买）
        active_subs.sort_by(|a, b| {
            a.billing_priority()
                .cmp(&b.billing_priority())
                .then(a.id.cmp(&b.id))
        });

        // 跳过 FREE，找第一个有消费的（current_credits < credit_limit）
        let current_sub = active_subs
            .iter()
            .filter(|s| s.plan_name.to_uppercase() != "FREE")
            .find(|s| s.current_credits < s.credit_limit);

        // 如果没找到有消费的，取第一个非 FREE 有余额的（fallback）
        let current_sub = current_sub.or_else(|| {
            active_subs
                .iter()
                .filter(|s| s.plan_name.to_uppercase() != "FREE")
                .find(|s| s.current_credits > 0.0)
        });

        // 构造 subscription_entity_list
        let subscription_entity_list: Vec<SubscriptionEntity> = active_subs
            .iter()
            .map(|s| SubscriptionEntity {
                subscription_name: s.plan_name.clone(),
                credit_limit: s.credit_limit,
                current_credits: s.current_credits,
                is_active: s.is_active,
            })
            .collect();

        // 获取当前套餐的数据
        let (credit_limit, current_credits) = match current_sub {
            Some(sub) => (sub.credit_limit, sub.current_credits),
            None => (0.0, 0.0),
        };

        let mut data = Code88UsageData {
            total_tokens: 0,
            credit_limit,
            current_credits,
            subscription_entity_list,
            used_tokens: 0,
            remaining_tokens: 0,
            percentage_used: 0.0,
        };

        // 计算 used_tokens, remaining_tokens, percentage_used
        data.calculate();
        data
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

    // 只要配置了 ANTHROPIC_BASE_URL，就返回对应的 auth_token
    // 支持所有中转站（88code、packy、以及其他第三方中转站）
    if env.base_url.is_some() {
        return env.auth_token;
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
    } else if base_url.contains("88code") {
        // 88code 中转站：只要 URL 包含 "88code" 就识别
        Some("https://www.88code.ai/api/usage".to_string())
    } else {
        // 其他中转站：基于 base_url 构造 usage URL
        // 假设 API 路径为 /api/usage/token/（与 Packy 兼容）
        let base = base_url.trim_end_matches('/');
        // 移除可能存在的 /v1 或 /api 后缀
        let base = base.trim_end_matches("/v1").trim_end_matches("/api");
        Some(format!("{}/api/usage/token/", base))
    }
}
