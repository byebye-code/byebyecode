use crate::api::{client::ApiClient, ApiConfig};
use crate::config::Config;
use crate::config::InputData;
use crate::core::segments::SegmentData;
use std::collections::HashMap;

/// 根据套餐类型获取语义化颜色
/// - PLUS/PRO/MAX: 橙色（高级套餐）
/// - PAYGO: 蓝色（按需套餐）
/// - FREE: 灰色（免费套餐）
/// - 其他: 白色
fn get_plan_color(plan_name: &str) -> &'static str {
    match plan_name.to_uppercase().as_str() {
        "PLUS" | "PRO" | "MAX" => "\x1b[38;5;214m", // 橙色
        "PAYGO" => "\x1b[38;5;39m",                 // 蓝色
        "FREE" => "\x1b[38;5;245m",                 // 灰色
        _ => "\x1b[38;5;255m",                      // 白色
    }
}

/// ANSI 重置代码
const RESET: &str = "\x1b[0m";

pub fn collect(config: &Config, input: &InputData) -> Option<SegmentData> {
    // Get API config from segment options
    let segment = config
        .segments
        .iter()
        .find(|s| matches!(s.id, crate::config::SegmentId::ByeByeCodeSubscription))?;

    if !segment.enabled {
        return None;
    }

    // Check if we are using Packy service
    // Priority:
    // 1. Current segment options
    // 2. ByeByeCodeUsage segment options (since users likely configure it there)
    // 3. Claude settings
    // 4. Default to 88code
    let usage_url = segment
        .options
        .get("usage_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            // Try to find usage_url in ByeByeCodeUsage segment options
            config
                .segments
                .iter()
                .find(|s| matches!(s.id, crate::config::SegmentId::ByeByeCodeUsage))
                .and_then(|s| s.options.get("usage_url"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
        .or_else(crate::api::get_usage_url_from_claude_settings)
        .unwrap_or_else(|| "https://www.88code.ai/api/usage".to_string());

    if usage_url.contains("packyapi.com") {
        return None;
    }

    // Try to get API key from segment options first, then from Claude settings
    let api_key = segment
        .options
        .get("api_key")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(crate::api::get_api_key_from_claude_settings);

    let api_key = match api_key {
        Some(key) if !key.is_empty() => key,
        _ => {
            return Some(SegmentData {
                primary: "未订阅".to_string(),
                secondary: String::new(),
                metadata: HashMap::new(),
            });
        }
    };

    // 实时获取数据，不使用缓存
    // 传入 model 参数以获取正确的套餐信息
    let model_id = &input.model.id;
    let subscriptions = fetch_subscriptions_sync(&api_key, Some(model_id))?;

    fn fetch_subscriptions_sync(
        api_key: &str,
        model: Option<&str>,
    ) -> Option<Vec<crate::api::SubscriptionData>> {
        let api_config = ApiConfig {
            enabled: true,
            api_key: api_key.to_string(),
            ..Default::default()
        };

        let client = ApiClient::new(api_config).ok()?;
        let subs = client.get_subscriptions(model).ok()?;
        Some(subs)
    }

    // 过滤掉已禁用的订阅和已过期的订阅（剩余天数 <= 0）
    let mut active_subscriptions: Vec<_> = subscriptions
        .iter()
        .filter(|sub| sub.is_active && sub.remaining_days > 0)
        .collect();

    // 排序：FREE 优先，然后按剩余天数升序
    active_subscriptions.sort_by(|a, b| {
        let a_is_free = a.plan_name.to_uppercase() == "FREE";
        let b_is_free = b.plan_name.to_uppercase() == "FREE";
        match (a_is_free, b_is_free) {
            (true, false) => std::cmp::Ordering::Less, // FREE 排前面
            (false, true) => std::cmp::Ordering::Greater, // 非FREE 排后面
            _ => a.remaining_days.cmp(&b.remaining_days), // 同类型按天数排
        }
    });

    if active_subscriptions.is_empty() {
        return Some(SegmentData {
            primary: "未订阅".to_string(),
            secondary: String::new(),
            metadata: HashMap::new(),
        });
    }

    // 组合所有订阅信息（精简格式）
    let mut subscription_texts = Vec::new();
    let mut metadata = HashMap::new();

    for (idx, sub) in active_subscriptions.iter().enumerate() {
        // 语义化颜色
        let color = get_plan_color(&sub.plan_name);

        // 精简格式：PLUS ¥198/月 53天
        // 去掉重置次数，只保留套餐名、价格、剩余天数
        let short_price = sub.plan_price.replace("付", "");
        let subscription_text = format!(
            "{}{} {} {}天{}",
            color, sub.plan_name, short_price, sub.remaining_days, RESET
        );
        subscription_texts.push(subscription_text);

        // 保存元数据
        metadata.insert(format!("plan_{}", idx), sub.plan_name.clone());
        metadata.insert(format!("price_{}", idx), sub.plan_price.clone());
        metadata.insert(format!("status_{}", idx), sub.status.clone());
        metadata.insert(format!("reset_times_{}", idx), sub.reset_times.to_string());
        metadata.insert(
            format!("remaining_days_{}", idx),
            sub.remaining_days.to_string(),
        );
        if let Some(expires) = &sub.expires_at {
            metadata.insert(format!("expires_at_{}", idx), expires.clone());
        }
    }

    // 用分隔符连接多个订阅
    let primary = subscription_texts.join(" | ");
    let secondary = String::new();

    Some(SegmentData {
        primary,
        secondary,
        metadata,
    })
}
