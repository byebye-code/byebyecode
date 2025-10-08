use crate::api::{client::ApiClient, ApiConfig};
use crate::config::Config;
use crate::config::InputData;
use crate::core::segments::SegmentData;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// 生成柔和的随机颜色（基于字符串哈希）
fn get_soft_color(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    // 定义一组柔和的颜色（RGB格式）
    let soft_colors = [
        (150, 180, 220), // 柔和蓝
        (180, 150, 200), // 柔和紫
        (200, 170, 150), // 柔和橙
        (150, 200, 180), // 柔和青
        (220, 180, 150), // 柔和棕
        (180, 200, 150), // 柔和绿
        (200, 150, 180), // 柔和粉
        (170, 190, 200), // 柔和灰蓝
    ];

    let idx = (hash % soft_colors.len() as u64) as usize;
    let (r, g, b) = soft_colors[idx];

    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

/// ANSI 重置代码
const RESET: &str = "\x1b[0m";

pub fn collect(config: &Config, _input: &InputData) -> Option<SegmentData> {
    // Get API config from segment options
    let segment = config
        .segments
        .iter()
        .find(|s| matches!(s.id, crate::config::SegmentId::ByeByeCodeSubscription))?;

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
                primary: "未订阅".to_string(),
                secondary: String::new(),
                metadata: HashMap::new(),
            });
        }
    };

    // 智能缓存策略
    let subscriptions =
        if let Some((mut cached_data, strategy)) = crate::api::cache::get_cached_subscriptions() {
            use crate::api::cache::CacheStrategy;

            match strategy {
                CacheStrategy::Valid => {
                    // 缓存有效，直接使用（需要格式化 plan_price）
                    for sub in &mut cached_data {
                        sub.format();
                    }
                    cached_data
                }
                CacheStrategy::StaleButUsable => {
                    // 缓存过期但可用，先返回旧数据，异步更新
                    crate::api::cache::spawn_background_subscription_update(api_key.clone());
                    for sub in &mut cached_data {
                        sub.format();
                    }
                    cached_data
                }
                CacheStrategy::MustRefresh => {
                    // 缓存太旧，必须立即刷新
                    fetch_subscriptions_sync(&api_key)?
                }
            }
        } else {
            // 没有缓存，立即获取
            fetch_subscriptions_sync(&api_key)?
        };

    fn fetch_subscriptions_sync(api_key: &str) -> Option<Vec<crate::api::SubscriptionData>> {
        let api_config = ApiConfig {
            enabled: true,
            api_key: api_key.to_string(),
            ..Default::default()
        };

        let client = ApiClient::new(api_config).ok()?;
        let subs = client.get_subscriptions().ok()?;
        let _ = crate::api::cache::save_cached_subscriptions(&subs);
        Some(subs)
    }

    // 过滤掉已禁用的订阅
    let active_subscriptions: Vec<_> = subscriptions.iter().filter(|sub| sub.is_active).collect();

    if active_subscriptions.is_empty() {
        return Some(SegmentData {
            primary: "未订阅".to_string(),
            secondary: String::new(),
            metadata: HashMap::new(),
        });
    }

    // 组合所有订阅信息
    let mut subscription_texts = Vec::new();
    let mut metadata = HashMap::new();

    for (idx, sub) in active_subscriptions.iter().enumerate() {
        // 构建每个订阅的完整信息
        let expiry_info = if sub.remaining_days >= 0 {
            format!("剩余{}天", sub.remaining_days)
        } else {
            "已过期".to_string()
        };

        // 为每个订阅生成基于其计划名的柔和颜色
        let color = get_soft_color(&sub.plan_name);

        // PAYGO 不显示重置次数，其他订阅显示
        let subscription_text = if sub.plan_name == "PAYGO" {
            format!(
                "{}{} {} ({}){}",
                color, sub.plan_name, sub.plan_price, expiry_info, RESET
            )
        } else {
            format!(
                "{}{} {} (可重置{}次, {}){}",
                color, sub.plan_name, sub.plan_price, sub.reset_times, expiry_info, RESET
            )
        };
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
