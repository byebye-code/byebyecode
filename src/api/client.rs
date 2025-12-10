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

    pub fn get_usage(&self, model: Option<&str>) -> Result<UsageData, Box<dyn std::error::Error>> {
        let is_packyapi = self.config.is_packyapi();

        let response = if is_packyapi {
            self.client
                .get(&self.config.usage_url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .send()?
        } else {
            // 构建请求体，传入 model 参数以获取正确套餐的用量
            // 如果不传 model，API 会默认返回 free 套餐的用量
            let body = match model {
                Some(m) => serde_json::json!({ "model": m }),
                None => serde_json::json!({}),
            };
            self.client
                .post(&self.config.usage_url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()?
        };

        if !response.status().is_success() {
            return Err(format!("Usage API request failed: {}", response.status()).into());
        }

        let response_text = response.text()?;

        let mut usage: UsageData = if is_packyapi {
            let resp: super::PackyUsageResponse =
                serde_json::from_str(&response_text).map_err(|e| {
                    format!(
                        "Packyapi JSON parse error: {} | Response: {}",
                        e, response_text
                    )
                })?;
            UsageData::Packy(resp.data)
        } else {
            // 解析 ResponseDTO 包装的响应
            let resp: super::ResponseDTO<super::Code88UsageData> =
                serde_json::from_str(&response_text).map_err(|e| {
                    format!("API JSON parse error: {} | Response: {}", e, response_text)
                })?;
            UsageData::Code88(resp.data)
        };

        usage.calculate();
        Ok(usage)
    }

    pub fn get_subscriptions(
        &self,
        model: Option<&str>,
    ) -> Result<Vec<SubscriptionData>, Box<dyn std::error::Error>> {
        // 构建请求体，传入 model 参数以获取正确的套餐信息
        // 如果不传 model，API 会默认返回 free 套餐
        let body = match model {
            Some(m) => serde_json::json!({ "model": m }),
            None => serde_json::json!({}),
        };

        let response = self
            .client
            .post(&self.config.subscription_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        if !response.status().is_success() {
            return Err(format!("Subscription API request failed: {}", response.status()).into());
        }

        let response_text = response.text()?;

        // 解析 ResponseDTO 包装的响应
        let resp: super::ResponseDTO<Vec<SubscriptionData>> = serde_json::from_str(&response_text)
            .map_err(|e| {
                format!(
                    "Subscription JSON parse error: {} | Response: {}",
                    e, response_text
                )
            })?;

        let mut subscriptions = resp.data;

        // 格式化每个订阅的显示数据
        for subscription in &mut subscriptions {
            subscription.format();
        }

        Ok(subscriptions)
    }

    pub fn check_token_limit(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // 这个方法用于快速检查，没有 model 上下文时传 None
        let usage = self.get_usage(None)?;
        Ok(usage.get_remaining_tokens() == 0)
    }
}
