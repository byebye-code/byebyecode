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
        .or_else(|| crate::api::get_api_key_from_claude_settings());

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

    match client.get_subscription() {
        Ok(sub) => {
            let expiry_info = if sub.remaining_days >= 0 {
                format!("剩余:{}天", sub.remaining_days)
            } else {
                "已过期".to_string()
            };

            let mut metadata = HashMap::new();
            metadata.insert("plan".to_string(), sub.plan_name.clone());
            metadata.insert("price".to_string(), sub.plan_price.clone());
            if let Some(expires) = &sub.expires_at {
                metadata.insert("expires_at".to_string(), expires.clone());
            }
            metadata.insert("active".to_string(), sub.is_active.to_string());

            Some(SegmentData {
                primary: format!("{} {}", sub.plan_name, sub.plan_price),
                secondary: format!("({})", expiry_info),
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
