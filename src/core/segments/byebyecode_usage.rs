use crate::config::Config;
use crate::config::InputData;
use crate::core::segments::SegmentData;
use crate::api::{client::ApiClient, ApiConfig};
use std::collections::HashMap;

pub fn collect(config: &Config, _input: &InputData) -> Option<SegmentData> {
    // Get API config from segment options
    let segment = config.segments.iter()
        .find(|s| matches!(s.id, crate::config::SegmentId::ByeByeCodeUsage))?;

    if !segment.enabled {
        return None;
    }

    // Try to get API key from segment options first, then from Claude settings
    let api_key = segment.options.get("api_key")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| crate::api::get_api_key_from_claude_settings());

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

    match client.get_usage() {
        Ok(usage) => {
            let used_dollars = usage.used_tokens as f64 / 100.0;
            let remaining_dollars = usage.remaining_tokens as f64 / 100.0;
            let total_dollars = usage.credit_limit;

            let mut metadata = HashMap::new();
            metadata.insert("used".to_string(), format!("{:.2}", used_dollars));
            metadata.insert("total".to_string(), format!("{:.2}", total_dollars));
            metadata.insert("remaining".to_string(), format!("{:.2}", remaining_dollars));

            Some(SegmentData {
                primary: format!("${:.2}/${:.0}", used_dollars, total_dollars),
                secondary: format!("剩${:.2}", remaining_dollars),
                metadata,
            })
        }
        Err(_) => {
            // API调用失败,显示错误信息
            Some(SegmentData {
                primary: format!("API错误"),
                secondary: String::new(),
                metadata: HashMap::new(),
            })
        }
    }
}
