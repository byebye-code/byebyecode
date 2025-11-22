use crate::api::{client::ApiClient, ApiConfig};
use crate::config::Config;
use crate::config::InputData;
use crate::core::segments::SegmentData;
use std::collections::HashMap;

pub fn collect(config: &Config, _input: &InputData) -> Option<SegmentData> {
    // Get API config from segment options
    let segment = config
        .segments
        .iter()
        .find(|s| matches!(s.id, crate::config::SegmentId::ByeByeCodeUsage))?;

    if !segment.enabled {
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
                primary: "未配置密钥".to_string(),
                secondary: String::new(),
                metadata: HashMap::new(),
            });
        }
    };

    let usage_url = segment
        .options
        .get("usage_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(crate::api::get_usage_url_from_claude_settings)
        .unwrap_or_else(|| "https://www.88code.org/api/usage".to_string());

    let subscription_url = segment
        .options
        .get("subscription_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "https://www.88code.org/api/subscription".to_string());

    let usage = fetch_usage_sync(&api_key, &usage_url)?;

    fn fetch_usage_sync(api_key: &str, usage_url: &str) -> Option<crate::api::UsageData> {
        let api_config = ApiConfig {
            enabled: true,
            api_key: api_key.to_string(),
            usage_url: usage_url.to_string(),
            subscription_url: String::new(),
        };

        let client = ApiClient::new(api_config).ok()?;
        let usage = client.get_usage().ok()?;
        Some(usage)
    }

    // 处理使用数据
    let used_dollars = usage.get_used_tokens() as f64 / 100.0;
    let remaining_dollars = (usage.get_remaining_tokens() as f64 / 100.0).max(0.0);
    let total_dollars = usage.get_credit_limit();

    let mut metadata = HashMap::new();
    metadata.insert("used".to_string(), format!("{:.2}", used_dollars));
    metadata.insert("total".to_string(), format!("{:.2}", total_dollars));
    metadata.insert("remaining".to_string(), format!("{:.2}", remaining_dollars));

    // 检查额度是否用完（包括超额使用）
    if usage.is_exhausted() {
        // 实时获取订阅信息
        let subscriptions = fetch_subscriptions_sync(&api_key, &subscription_url);

        if let Some(subs) = subscriptions {
            let active_subs: Vec<_> = subs.iter().filter(|s| s.is_active).collect();

            if active_subs.len() > 1 {
                // 有多个订阅，提示切换到其他套餐
                return Some(SegmentData {
                    primary: format!("${:.2}/${:.0} 已用完", used_dollars, total_dollars),
                    secondary: "提示：你有其他套餐可用".to_string(),
                    metadata,
                });
            } else if active_subs.len() == 1 {
                // 只有一个订阅，提示手动重置
                let reset_times = active_subs[0].reset_times;
                if reset_times > 0 {
                    return Some(SegmentData {
                        primary: format!("${:.2}/${:.0} 已用完", used_dollars, total_dollars),
                        secondary: format!("可重置{}次，请手动重置", reset_times),
                        metadata,
                    });
                } else {
                    return Some(SegmentData {
                        primary: format!("${:.2}/${:.0} 已用完", used_dollars, total_dollars),
                        secondary: "无可用重置次数".to_string(),
                        metadata,
                    });
                }
            }
        }

        // 没有订阅信息或无活跃订阅，显示基本提示
        return Some(SegmentData {
            primary: format!("${:.2}/${:.0} 已用完", used_dollars, total_dollars),
            secondary: "请充值或重置额度".to_string(),
            metadata,
        });
    }

    // 正常显示
    Some(SegmentData {
        primary: format!("${:.2}/${:.0}", used_dollars, total_dollars),
        secondary: format!("剩${:.2}", remaining_dollars),
        metadata,
    })
}

fn fetch_subscriptions_sync(
    api_key: &str,
    subscription_url: &str,
) -> Option<Vec<crate::api::SubscriptionData>> {
    let api_config = ApiConfig {
        enabled: true,
        api_key: api_key.to_string(),
        usage_url: String::new(),
        subscription_url: subscription_url.to_string(),
    };

    let client = ApiClient::new(api_config).ok()?;
    let subs = client.get_subscriptions().ok()?;
    Some(subs)
}
