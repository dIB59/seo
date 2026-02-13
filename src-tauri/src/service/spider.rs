use anyhow::{Context, Result};
use rquest::Client;
use rquest_util::Emulation;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum ClientType {
    Standard,
    HeavyEmulation,
}

/// A wrapper around the HTTP client to avoid leaking `rquest` types.
pub struct Spider {
    client: Client,
}

impl Spider {
    pub fn new(client_type: ClientType) -> Result<Self> {
        let builder = Client::builder().timeout(Duration::from_secs(30));

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

    /// Fetch HTML content from a URL.
    pub async fn fetch_html(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        Ok(body)
    }

    /// Fetch a URL and return a generic response DTO.
    pub async fn get(&self, url: &str) -> Result<SpiderResponse> {
        let response = self.client.get(url).send().await?;
        let status = response.status().as_u16();
        let body = response.text().await?;
        Ok(SpiderResponse {
            status,
            body,
            url: url.to_string(),
        })
    }

    /// POST JSON to a URL.
    pub async fn post_json<T: serde::Serialize>(
        &self,
        url: &str,
        payload: &T,
    ) -> Result<SpiderResponse> {
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

pub struct SpiderResponse {
    pub status: u16,
    pub body: String,
    pub url: String,
}
