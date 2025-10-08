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

    // 直接调用API获取实时数据(无缓存)
    let api_config = ApiConfig {
        enabled: true,
        api_key,
        ..Default::default()
    };

    let client = match ApiClient::new(api_config) {
        Ok(c) => c,
        Err(_) => {
            return Some(SegmentData {
                primary: "客户端错误".to_string(),
                secondary: String::new(),
                metadata: HashMap::new(),
            });
        }
    };

    match client.get_subscriptions() {
        Ok(subscriptions) => {
            if subscriptions.is_empty() {
                return Some(SegmentData {
                    primary: "未订阅".to_string(),
                    secondary: String::new(),
                    metadata: HashMap::new(),
                });
            }

            // 组合所有订阅信息
            let mut subscription_texts = Vec::new();
            let mut metadata = HashMap::new();

            for (idx, sub) in subscriptions.iter().enumerate() {
                // 构建每个订阅的完整信息: PAYGO ¥29.9/年付 (活跃中, 可重置2次, 剩余365天)
                let status_text = if sub.is_active {
                    "活跃中"
                } else {
                    "已禁用"
                };

                let expiry_info = if sub.remaining_days >= 0 {
                    format!("剩余{}天", sub.remaining_days)
                } else {
                    "已过期".to_string()
                };

                // 为每个订阅生成基于其计划名的柔和颜色
                let color = get_soft_color(&sub.plan_name);
                let subscription_text = format!(
                    "{}{} {} ({}, 可重置{}次, {}){}",
                    color,
                    sub.plan_name,
                    sub.plan_price,
                    status_text,
                    sub.reset_times,
                    expiry_info,
                    RESET
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
        Err(_) => {
            // API调用失败,显示错误信息
            Some(SegmentData {
                primary: "API错误".to_string(),
                secondary: String::new(),
                metadata: HashMap::new(),
            })
        }
    }
}
