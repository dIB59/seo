use anyhow::{Context, Result};
use async_trait::async_trait;
use rquest::Client;
use rquest_util::Emulation;
use serde::Serialize;
use std::pin::Pin;
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

    /// Begin a streaming GET for large binary downloads (e.g. model files).
    /// Returns a [`StreamResponse`] whose [`StreamResponse::next_chunk`] method
    /// yields successive byte chunks until the body is exhausted.
    async fn stream_get(&self, url: &str) -> Result<StreamResponse>;
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

    async fn stream_get(&self, url: &str) -> Result<StreamResponse> {
        let response = self.client.get(url).send().await?;
        let status = response.status().as_u16();
        let content_length = response.content_length();
        Ok(StreamResponse::new(
            status,
            content_length,
            Box::new(RquestChunker(response)),
        ))
    }
}

// ── StreamResponse ────────────────────────────────────────────────────────────

/// Type-erased streaming response for large binary downloads.
/// All `rquest` types are contained here; callers only see `Vec<u8>` chunks.
pub struct StreamResponse {
    pub status: u16,
    /// `None` when the server omits `Content-Length`.
    pub content_length: Option<u64>,
    inner: Box<dyn ChunkStream>,
}

impl StreamResponse {
    fn new(status: u16, content_length: Option<u64>, inner: Box<dyn ChunkStream>) -> Self {
        Self { status, content_length, inner }
    }

    pub async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>> {
        self.inner.next_chunk().await
    }
}

// Object-safe async chunk iterator — keeps rquest types private to this module.
trait ChunkStream: Send {
    fn next_chunk<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send + 'a>>;
}

struct RquestChunker(rquest::Response);

impl ChunkStream for RquestChunker {
    fn next_chunk<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send + 'a>> {
        Box::pin(async move {
            Ok(self.0.chunk().await?.map(|b| b.to_vec()))
        })
    }
}

// ── SpiderResponse ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct SpiderResponse {
    pub status: u16,
    pub body: String,
    pub url: String,
}

// ── MockSpider (test only) ────────────────────────────────────────────────────

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

    async fn stream_get(&self, _url: &str) -> Result<StreamResponse> {
        Ok(StreamResponse::new(200, Some(0), Box::new(EmptyChunker)))
    }
}

#[cfg(test)]
struct EmptyChunker;

#[cfg(test)]
impl ChunkStream for EmptyChunker {
    fn next_chunk<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Option<Vec<u8>>>> + Send + 'a>> {
        Box::pin(async move { Ok(None) })
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
