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

    // 智能缓存策略（usage 更频繁更新）
    let usage = if let Some((cached_data, strategy)) = crate::api::cache::get_cached_usage() {
        use crate::api::cache::CacheStrategy;

        match strategy {
            CacheStrategy::Valid => {
                // 缓存有效（1分钟内），直接使用
                cached_data
            }
            CacheStrategy::StaleButUsable => {
                // 缓存过期但可用（1分钟到1小时），先返回旧数据，异步更新
                crate::api::cache::spawn_background_usage_update(api_key.clone());
                cached_data
            }
            CacheStrategy::MustRefresh => {
                // 缓存太旧（>1小时），必须立即刷新
                fetch_usage_sync(&api_key)?
            }
        }
    } else {
        // 没有缓存，立即获取
        fetch_usage_sync(&api_key)?
    };

    fn fetch_usage_sync(api_key: &str) -> Option<crate::api::UsageData> {
        let api_config = ApiConfig {
            enabled: true,
            api_key: api_key.to_string(),
            ..Default::default()
        };

        let client = ApiClient::new(api_config).ok()?;
        let usage = client.get_usage().ok()?;
        let _ = crate::api::cache::save_cached_usage(&usage);
        Some(usage)
    }

    // 处理使用数据
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
