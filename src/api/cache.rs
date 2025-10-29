use super::{SubscriptionData, UsageData};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

/// 缓存有效期：5分钟
const CACHE_FRESH_SECONDS: u64 = 300;

/// 获取缓存文件路径
fn get_cache_file(cache_type: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let cache_dir = home.join(".claude").join("byebyecode").join("cache");

    // 确保缓存目录存在
    fs::create_dir_all(&cache_dir).ok()?;

    Some(cache_dir.join(format!("{}.json", cache_type)))
}

/// 检查缓存是��新鲜（5分钟内）
fn is_cache_fresh(cache_file: &PathBuf) -> bool {
    if let Ok(metadata) = fs::metadata(cache_file) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                return elapsed.as_secs() < CACHE_FRESH_SECONDS;
            }
        }
    }
    false
}

/// 读取缓存文件
fn read_cache<T: serde::de::DeserializeOwned>(cache_file: &PathBuf) -> Option<T> {
    let content = fs::read_to_string(cache_file).ok()?;
    serde_json::from_str(&content).ok()
}

/// 保存缓存文件（覆盖旧缓存）
fn save_cache<T: serde::Serialize>(cache_file: &PathBuf, data: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(data)?;
    fs::write(cache_file, json)?;
    Ok(())
}

/// 获取订阅数据缓存
/// 返回: (缓存数据, 是否需要延迟刷新)
/// - 5分钟内：返回缓存，不需要刷新
/// - 5分钟外：返回缓存，需要延迟1秒后台刷新
pub fn get_cached_subscriptions() -> (Option<Vec<SubscriptionData>>, bool) {
    let cache_file = match get_cache_file("subscriptions") {
        Some(f) => f,
        None => return (None, false),
    };

    // 读取缓存
    let cached_data: Option<Vec<SubscriptionData>> = read_cache(&cache_file);

    if cached_data.is_none() {
        // 没有缓存，需要立即获取
        return (None, false);
    }

    // 检查缓存新鲜度
    let is_fresh = is_cache_fresh(&cache_file);

    // 返回缓存数据 + 是否需要延迟刷新（5分钟外需要刷新）
    (cached_data, !is_fresh)
}

/// 保存订阅数据到缓存（覆盖旧缓存）
pub fn save_cached_subscriptions(data: &Vec<SubscriptionData>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(cache_file) = get_cache_file("subscriptions") {
        save_cache(&cache_file, data)?;
    }
    Ok(())
}

/// 获取使用量数据缓存
/// 返回: (缓存数据, 是否需要延迟刷新)
pub fn get_cached_usage() -> (Option<UsageData>, bool) {
    let cache_file = match get_cache_file("usage") {
        Some(f) => f,
        None => return (None, false),
    };

    // 读取缓存
    let cached_data: Option<UsageData> = read_cache(&cache_file);

    if cached_data.is_none() {
        return (None, false);
    }

    // 检查缓存新鲜度
    let is_fresh = is_cache_fresh(&cache_file);

    (cached_data, !is_fresh)
}

/// 保存使用量数据到缓存（覆盖旧缓存）
pub fn save_cached_usage(data: &UsageData) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(cache_file) = get_cache_file("usage") {
        save_cache(&cache_file, data)?;
    }
    Ok(())
}

/// 后台异步更新订阅数据（延迟1秒执行）
pub fn spawn_background_subscription_update(api_key: String) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let api_config = super::ApiConfig {
            enabled: true,
            api_key,
            subscription_url: "https://api.cometix.cn/v1/billing/subscription/list".to_string(),
            usage_url: "https://api.byebyecode.tech/v1/balance/remaining".to_string(),
        };

        if let Ok(client) = super::client::ApiClient::new(api_config) {
            if let Ok(subs) = client.get_subscriptions() {
                let _ = save_cached_subscriptions(&subs);
            }
        }
    });
}

/// 后台异步更新使用量数据（延迟1秒执行）
pub fn spawn_background_usage_update(api_key: String) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let api_config = super::ApiConfig {
            enabled: true,
            api_key,
            subscription_url: "https://api.cometix.cn/v1/billing/subscription/list".to_string(),
            usage_url: "https://api.byebyecode.tech/v1/balance/remaining".to_string(),
        };

        if let Ok(client) = super::client::ApiClient::new(api_config) {
            if let Ok(usage) = client.get_usage() {
                let _ = save_cached_usage(&usage);
            }
        }
    });
}
