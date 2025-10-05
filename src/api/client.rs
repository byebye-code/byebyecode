use super::{ApiConfig, UsageData, SubscriptionData};
use reqwest::blocking::Client;
use std::time::Duration;

pub struct ApiClient {
    config: ApiConfig,
    client: Client,
}

impl ApiClient {
    pub fn new(config: ApiConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("byebyecode/1.0.0")
            .build()?;

        Ok(Self { config, client })
    }

    pub fn get_usage(&self) -> Result<UsageData, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&self.config.usage_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Usage API request failed: {}", response.status()).into());
        }

        let mut usage: UsageData = response.json()?;
        usage.calculate(); // 计算使用情况
        Ok(usage)
    }

    pub fn get_subscription(&self) -> Result<SubscriptionData, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&self.config.subscription_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Subscription API request failed: {}", response.status()).into());
        }

        // API返回的是数组,取第一个活跃的订阅
        let subscriptions: Vec<SubscriptionData> = response.json()?;

        let mut subscription = subscriptions
            .into_iter()
            .find(|s| s.status == "活跃中")
            .ok_or("No active subscription found")?;

        subscription.format(); // 格式化显示数据
        Ok(subscription)
    }

    pub fn check_token_limit(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let usage = self.get_usage()?;
        Ok(usage.remaining_tokens == 0)
    }
}
