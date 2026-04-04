use chrono::{DateTime, Utc};
use serde::Serialize;

/// Pages with load time ≤ 2s considered mobile-friendly (speed heuristic fallback).
const SPEED_HEURISTIC_LOAD_TIME_MS: i64 = 2000;

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: i64,
    pub status_code: Option<i64>,
    pub content_type: Option<String>,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub canonical_url: Option<String>,
    pub robots_meta: Option<String>,
    pub word_count: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub response_size_bytes: Option<i64>,
    pub has_viewport: bool,
    pub has_structured_data: bool,
    pub crawled_at: DateTime<Utc>,
    /// Extracted data from custom extractors (key-value pairs)
    #[serde(default)]
    pub extracted_data: std::collections::HashMap<String, serde_json::Value>,
}

impl Page {
    pub fn is_mobile_friendly_heuristic(&self) -> bool {
        self.has_viewport
            && self
                .load_time_ms
                .is_some_and(|t| t <= SPEED_HEURISTIC_LOAD_TIME_MS)
    }

}

/// Lightweight page info for listings.
#[derive(Debug, Clone, Serialize)]
pub struct PageInfo {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub issue_count: i64,
}

/// Status of a page in the analysis queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageQueueStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl PageQueueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for PageQueueStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for PageQueueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A page item in the analysis queue.
/// Tracks pages to be analyzed, enabling resumability and concurrent processing.
#[derive(Debug, Clone, Serialize)]
pub struct PageQueueItem {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: i64,
    pub status: PageQueueStatus,
    pub retry_count: i64,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PageQueueItem {
    /// Create a new pending page queue item.
    pub fn new(job_id: &str, url: &str, depth: i64) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_id: job_id.to_string(),
            url: url.to_string(),
            depth,
            status: PageQueueStatus::Pending,
            retry_count: 0,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new pending page queue item with a specific ID.
    pub fn with_id(id: &str, job_id: &str, url: &str, depth: i64) -> Self {
        let mut item = Self::new(job_id, url, depth);
        item.id = id.to_string();
        item
    }

    /// Mark the item as processing.
    pub fn mark_processing(&mut self) {
        self.status = PageQueueStatus::Processing;
        self.updated_at = Utc::now();
    }

    /// Mark the item as completed.
    pub fn mark_completed(&mut self) {
        self.status = PageQueueStatus::Completed;
        self.updated_at = Utc::now();
    }

    /// Mark the item as failed with an error message.
    pub fn mark_failed(&mut self, error: &str) {
        self.status = PageQueueStatus::Failed;
        self.error_message = Some(error.to_string());
        self.retry_count += 1;
        self.updated_at = Utc::now();
    }

    /// Check if the item can be retried.
    pub fn can_retry(&self, max_retries: i64) -> bool {
        self.retry_count < max_retries && self.status == PageQueueStatus::Failed
    }
}

/// New page queue item for insertion.
#[derive(Debug, Clone)]
pub struct NewPageQueueItem {
    pub job_id: String,
    pub url: String,
    pub depth: i64,
}

impl NewPageQueueItem {
    pub fn new(job_id: &str, url: &str, depth: i64) -> Self {
        Self {
            job_id: job_id.to_string(),
            url: url.to_string(),
            depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_page(overrides: impl FnOnce(&mut Page)) -> Page {
        let mut page = Page {
            id: "1".to_string(),
            job_id: "job1".to_string(),
            url: "https://example.com".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: None,
            title: Some("Title".to_string()),
            meta_description: Some("desc".to_string()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(1000),
            response_size_bytes: Some(512),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        };
        overrides(&mut page);
        page
    }

    #[test]
    fn test_mobile_friendly_heuristic_fast_page() {
        let page = make_page(|p| p.load_time_ms = Some(1500));
        assert!(page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn test_mobile_friendly_heuristic_slow_page() {
        let page = make_page(|p| p.load_time_ms = Some(3000));
        assert!(!page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn test_mobile_friendly_heuristic_exactly_threshold() {
        let page = make_page(|p| p.load_time_ms = Some(2000));
        assert!(page.is_mobile_friendly_heuristic());
    }
}