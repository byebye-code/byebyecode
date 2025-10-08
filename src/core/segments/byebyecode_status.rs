use crate::config::Config;
use crate::config::InputData;
use crate::core::segments::SegmentData;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const HEALTH_CHECK_URL: &str = "https://www.88code.org";
const CACHE_TTL_SECONDS: u64 = 30; // 30秒检查一次健康状态

/// 获取缓存路径
fn get_health_cache_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| {
        home.join(".claude/88code")
            .join(".cache")
            .join("88code_health.json")
    })
}

/// 健康状态缓存
#[derive(serde::Deserialize, serde::Serialize)]
struct HealthCache {
    is_healthy: bool,
    timestamp: u64,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 获取进程ID作为随机种子
fn get_process_id() -> u32 {
    std::process::id()
}

/// 根据进程ID生成一个固定的颜色索引
/// 每次启动进程会得到不同的颜色,但同一次运行中颜色保持不变
fn get_random_color() -> u8 {
    let colors = get_wave_colors();
    let pid = get_process_id();
    // 使用进程ID对颜色数量取模,得到一个固定的颜色索引
    colors[(pid as usize) % colors.len()]
}

/// 读取健康状态缓存
fn read_health_cache() -> Option<bool> {
    let path = get_health_cache_path()?;
    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(path).ok()?;
    let cache: HealthCache = serde_json::from_str(&content).ok()?;

    let now = current_timestamp();
    if now - cache.timestamp < CACHE_TTL_SECONDS {
        Some(cache.is_healthy)
    } else {
        None
    }
}

/// 写入健康状态缓存
fn write_health_cache(is_healthy: bool) {
    if let Some(path) = get_health_cache_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let cache = HealthCache {
            is_healthy,
            timestamp: current_timestamp(),
        };

        if let Ok(content) = serde_json::to_string(&cache) {
            let _ = std::fs::write(path, content);
        }
    }
}

/// 检查88code服务健康状态
fn check_health() -> bool {
    use reqwest::blocking::Client;
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("byebyecode/1.0.0")
        .danger_accept_invalid_certs(false) // 接受有效证书
        .build();

    if let Ok(client) = client {
        match client.get(HEALTH_CHECK_URL).send() {
            Ok(response) => {
                let status = response.status().as_u16();
                // 200-299 都认为是健康的
                return status >= 200 && status < 300;
            }
            Err(_e) => {
                // 网络错误时，默认认为服务是健康的（避免误报）
                // 只有明确收到错误响应时才认为不健康
                return true;
            }
        }
    }

    // 客户端构建失败，默认健康
    true
}

/// 获取可用的颜色列表
fn get_wave_colors() -> Vec<u8> {
    // 彩虹色相环: 红-橙-黄-绿-青-蓝-紫
    // 使用256色模式的颜色码
    vec![
        196, // 红
        202, // 橙
        226, // 黄
        46,  // 绿
        51,  // 青
        21,  // 蓝
        129, // 紫
    ]
}

pub fn collect(_config: &Config, _input: &InputData) -> Option<SegmentData> {
    // 先检查缓存
    let is_healthy = if let Some(cached) = read_health_cache() {
        cached
    } else {
        // 缓存过期,重新检查
        let health = check_health();
        write_health_cache(health);
        health
    };

    let mut metadata = HashMap::new();
    metadata.insert("healthy".to_string(), is_healthy.to_string());

    if is_healthy {
        // 服务正常,显示静态彩色文本
        // 每次进程启动时颜色不同,但同一次运行中保持固定
        let message = "88code正持续为您服务";
        let color = get_random_color();

        Some(SegmentData {
            primary: format!("\x1b[38;5;{}m{}\x1b[0m", color, message),
            secondary: String::new(),
            metadata,
        })
    } else {
        // 服务断开,显示红色警告
        Some(SegmentData {
            primary: "\x1b[38;5;196m88code服务断开\x1b[0m".to_string(),
            secondary: String::new(),
            metadata,
        })
    }
}
