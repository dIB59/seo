use anyhow::{Context, Result};
use async_trait::async_trait;
use rquest::Client;
use rquest_util::Emulation;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize, specta::Type)]
pub enum ClientType {
    Standard,
    HeavyEmulation,
}

#[async_trait]
pub trait SpiderAgent: Send + Sync {
    async fn fetch_html(&self, url: &str) -> Result<String>;

    async fn get(&self, url: &str) -> Result<SpiderResponse>;

    async fn post_json(&self, url: &str, payload: &serde_json::Value) -> Result<SpiderResponse>;
}

pub struct Spider {
    client: Client,
}

impl Spider {
    pub fn new(client_type: ClientType) -> Result<Self> {
        let builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(rquest::redirect::Policy::limited(10));

        let client = match client_type {
            ClientType::HeavyEmulation => builder
                .emulation(Emulation::Firefox136)
                .build()
                .context("Failed to build heavy impersonated rquest client")?,
            ClientType::Standard => builder
                .build()
                .context("Failed to build standard rquest client")?,
        };

        Ok(Self { client })
    }

    pub fn new_agent(client_type: ClientType) -> Result<Arc<dyn SpiderAgent>> {
        Ok(Arc::new(Self::new(client_type)?))
    }
}

#[async_trait]
impl SpiderAgent for Spider {
    async fn fetch_html(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        Ok(body)
    }

    async fn get(&self, url: &str) -> Result<SpiderResponse> {
        let response = self.client.get(url).send().await?;
        let status = response.status().as_u16();
        let final_url = response.url().to_string();

        if final_url != url {
            tracing::info!("[SPIDER] Redirected: {} -> {}", url, final_url);
        }

        let body = response.text().await?;
        Ok(SpiderResponse {
            status,
            body,
            url: final_url,
        })
    }

    async fn post_json(&self, url: &str, payload: &serde_json::Value) -> Result<SpiderResponse> {
        let response = self.client.post(url).json(payload).send().await?;
        let status = response.status().as_u16();
        let body = response.text().await?;
        Ok(SpiderResponse {
            status,
            body,
            url: url.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct SpiderResponse {
    pub status: u16,
    pub body: String,
    pub url: String,
}
#[cfg(test)]
pub struct MockSpider {
    pub html_response: String,
    pub generic_response: SpiderResponse,
}

#[cfg(test)]
#[async_trait]
impl SpiderAgent for MockSpider {
    async fn fetch_html(&self, _url: &str) -> Result<String> {
        Ok(self.html_response.clone())
    }

    async fn get(&self, _url: &str) -> Result<SpiderResponse> {
        Ok(self.generic_response.clone())
    }

    async fn post_json(&self, _url: &str, _payload: &serde_json::Value) -> Result<SpiderResponse> {
        Ok(self.generic_response.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_spider() {
        let mock = MockSpider {
            html_response: "<html></html>".to_string(),
            generic_response: SpiderResponse {
                status: 200,
                body: "ok".to_string(),
                url: "test".to_string(),
            },
        };

        let agent: Arc<dyn SpiderAgent> = Arc::new(mock);
        assert_eq!(agent.fetch_html("h").await.unwrap(), "<html></html>");
    }
}
