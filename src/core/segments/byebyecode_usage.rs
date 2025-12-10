use crate::api::{client::ApiClient, ApiConfig};
use crate::config::Config;
use crate::config::InputData;
use crate::core::segments::SegmentData;
use std::collections::HashMap;

/// ANSI 重置代码
const RESET: &str = "\x1b[0m";

/// 根据百分比获取状态色（柔和色调）
/// - 0-50%: 柔和绿 (充足)
/// - 50-80%: 柔和黄 (注意)
/// - 80%+: 柔和红 (紧急)
fn get_status_color(percentage: f64) -> &'static str {
    if percentage <= 50.0 {
        "\x1b[38;5;114m" // 柔和绿 (256色 #114)
    } else if percentage <= 80.0 {
        "\x1b[38;5;179m" // 柔和黄/橙 (256色 #179)
    } else {
        "\x1b[38;5;174m" // 柔和红/粉 (256色 #174)
    }
}

pub fn collect(config: &Config, input: &InputData) -> Option<SegmentData> {
    // Get API config from segment options
    let segment = config
        .segments
        .iter()
        .find(|s| matches!(s.id, crate::config::SegmentId::ByeByeCodeUsage))?;

    if !segment.enabled {
        return None;
    }

    let usage_url = segment
        .options
        .get("usage_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(crate::api::get_usage_url_from_claude_settings)
        .unwrap_or_else(|| "https://www.88code.ai/api/usage".to_string());

    // 根据 usage_url 判断是哪个服务，并设置动态图标
    let service_name = if usage_url.contains("packyapi.com") {
        "packy"
    } else {
        "88code"
    };

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
            let mut metadata = HashMap::new();
            metadata.insert("dynamic_icon".to_string(), service_name.to_string());
            return Some(SegmentData {
                primary: "未配置密钥".to_string(),
                secondary: String::new(),
                metadata,
            });
        }
    };

    let subscription_url = segment
        .options
        .get("subscription_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "https://www.88code.ai/api/subscription".to_string());

    // 从输入数据获取当前使用的模型
    let model_id = &input.model.id;
    let usage = fetch_usage_sync(&api_key, &usage_url, Some(model_id))?;

    fn fetch_usage_sync(
        api_key: &str,
        usage_url: &str,
        model: Option<&str>,
    ) -> Option<crate::api::UsageData> {
        let api_config = ApiConfig {
            enabled: true,
            api_key: api_key.to_string(),
            usage_url: usage_url.to_string(),
            subscription_url: String::new(),
        };

        let client = ApiClient::new(api_config).ok()?;
        let usage = client.get_usage(model).ok()?;
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
    metadata.insert("service".to_string(), service_name.to_string());
    metadata.insert("dynamic_icon".to_string(), service_name.to_string());

    // 检查额度是否用完（包括超额使用）
    if usage.is_exhausted() {
        // 实时获取订阅信息，传入 model 以获取正确的套餐
        let model_id = &input.model.id;
        let subscriptions = fetch_subscriptions_sync(&api_key, &subscription_url, Some(model_id));

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

    // 正常显示 - 使用进度条可视化
    let percentage = if total_dollars > 0.0 {
        (used_dollars / total_dollars * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    // 生成进度条（10格）+ 状态色
    let bar_length = 10;
    let filled = ((percentage / 100.0) * bar_length as f64).round() as usize;
    let empty = bar_length - filled;

    // 根据百分比获取状态色
    let status_color = get_status_color(percentage);
    let progress_bar = format!(
        "{}{}{}{}",
        status_color,
        "▓".repeat(filled),
        "░".repeat(empty),
        RESET
    );

    Some(SegmentData {
        primary: format!(
            "${:.2}/${:.0} {}",
            used_dollars, total_dollars, progress_bar
        ),
        secondary: String::new(),
        metadata,
    })
}

fn fetch_subscriptions_sync(
    api_key: &str,
    subscription_url: &str,
    model: Option<&str>,
) -> Option<Vec<crate::api::SubscriptionData>> {
    let api_config = ApiConfig {
        enabled: true,
        api_key: api_key.to_string(),
        usage_url: String::new(),
        subscription_url: subscription_url.to_string(),
    };

    let client = ApiClient::new(api_config).ok()?;
    let subs = client.get_subscriptions(model).ok()?;
    Some(subs)
}
