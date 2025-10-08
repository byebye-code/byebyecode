use super::{ApiConfig, SubscriptionData, UsageData};
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
        let response = self
            .client
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

    pub fn get_subscriptions(&self) -> Result<Vec<SubscriptionData>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(&self.config.subscription_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Subscription API request failed: {}", response.status()).into());
        }

        // API返回的是数组,返回所有订阅
        let mut subscriptions: Vec<SubscriptionData> = response.json()?;

        // 格式化每个订阅的显示数据
        for subscription in &mut subscriptions {
            subscription.format();
        }

        Ok(subscriptions)
    }

    pub fn check_token_limit(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let usage = self.get_usage()?;
        Ok(usage.remaining_tokens == 0)
    }
}
