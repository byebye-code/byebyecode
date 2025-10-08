use super::{SubscriptionData, UsageData};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_DIR: &str = ".claude/ccline/cache";
const SUBSCRIPTION_TTL_SECONDS: u64 = 1800; // 30分钟缓存
const USAGE_TTL_SECONDS: u64 = 60; // 1分钟缓存
const FORCE_REFRESH_THRESHOLD: u64 = 3600; // 1小时阈值，超过则强制刷新

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheStrategy {
    /// 缓存有效，直接使用
    Valid,
    /// 缓存过期但在快速启动窗口内（5分钟），先用旧缓存，异步更新
    StaleButUsable,
    /// 缓存很旧（>1小时）或不存在，必须立即刷新
    MustRefresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedData<T> {
    data: T,
    timestamp: u64,
}

impl<T> CachedData<T> {
    fn new(data: T) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self { data, timestamp }
    }

    fn get_age(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp
    }

    fn get_strategy(&self, ttl_seconds: u64) -> CacheStrategy {
        let age = self.get_age();

        if age < ttl_seconds {
            // 缓存仍然有效
            CacheStrategy::Valid
        } else if age < FORCE_REFRESH_THRESHOLD {
            // 缓存过期但未超过1小时，可以先用旧的
            CacheStrategy::StaleButUsable
        } else {
            // 缓存太旧，必须刷新
            CacheStrategy::MustRefresh
        }
    }
}

fn get_cache_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(CACHE_DIR))
}

fn get_cache_path(filename: &str) -> Option<PathBuf> {
    get_cache_dir().map(|dir| dir.join(filename))
}

fn ensure_cache_dir() -> std::io::Result<()> {
    if let Some(dir) = get_cache_dir() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// 读取缓存的订阅数据，返回数据和缓存策略
pub fn get_cached_subscriptions() -> Option<(Vec<SubscriptionData>, CacheStrategy)> {
    let path = get_cache_path("subscriptions.json")?;
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    let cached: CachedData<Vec<SubscriptionData>> = serde_json::from_str(&content).ok()?;

    let strategy = cached.get_strategy(SUBSCRIPTION_TTL_SECONDS);
    Some((cached.data, strategy))
}

/// 保存订阅数据到缓存
pub fn save_cached_subscriptions(data: &[SubscriptionData]) -> std::io::Result<()> {
    ensure_cache_dir()?;
    let path = get_cache_path("subscriptions.json")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cache path not found"))?;

    let cached = CachedData::new(data.to_vec());
    let content = serde_json::to_string(&cached)?;
    fs::write(path, content)?;
    Ok(())
}

/// 读取缓存的使用数据，返回数据和缓存策略
pub fn get_cached_usage() -> Option<(UsageData, CacheStrategy)> {
    let path = get_cache_path("usage.json")?;
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    let cached: CachedData<UsageData> = serde_json::from_str(&content).ok()?;

    let strategy = cached.get_strategy(USAGE_TTL_SECONDS);
    Some((cached.data, strategy))
}

/// 保存使用数据到缓存
pub fn save_cached_usage(data: &UsageData) -> std::io::Result<()> {
    ensure_cache_dir()?;
    let path = get_cache_path("usage.json")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cache path not found"))?;

    let cached = CachedData::new(data.clone());
    let content = serde_json::to_string(&cached)?;
    fs::write(path, content)?;
    Ok(())
}

/// 异步后台更新订阅数据（延迟1秒启动）
pub fn spawn_background_subscription_update(api_key: String) {
    std::thread::spawn(move || {
        // 延迟1秒以不阻塞主线程
        std::thread::sleep(std::time::Duration::from_secs(1));

        let api_config = super::ApiConfig {
            enabled: true,
            api_key,
            ..Default::default()
        };

        if let Ok(client) = super::client::ApiClient::new(api_config) {
            if let Ok(subs) = client.get_subscriptions() {
                let _ = save_cached_subscriptions(&subs);
            }
        }
    });
}

/// 异步后台更新使用数据（延迟1秒启动）
pub fn spawn_background_usage_update(api_key: String) {
    std::thread::spawn(move || {
        // 延迟1秒以不阻塞主线程
        std::thread::sleep(std::time::Duration::from_secs(1));

        let api_config = super::ApiConfig {
            enabled: true,
            api_key,
            ..Default::default()
        };

        if let Ok(client) = super::client::ApiClient::new(api_config) {
            if let Ok(usage) = client.get_usage() {
                let _ = save_cached_usage(&usage);
            }
        }
    });
}
