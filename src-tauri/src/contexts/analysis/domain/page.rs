use super::depth::Depth;
use super::retry_count::RetryCount;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Pages with load time ≤ 2s considered mobile-friendly (speed heuristic fallback).
const SPEED_HEURISTIC_LOAD_TIME_MS: i64 = 2000;

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: Depth,
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

/// Error returned when [`PageQueueStatus::from_str`] receives an
/// unrecognised string. Replaces the previous `Err = ()` so consumers
/// have a typed error and a useful Display message instead of a
/// silent unit.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid page queue status: '{0}'")]
pub struct ParsePageQueueStatusError(pub String);

impl std::str::FromStr for PageQueueStatus {
    type Err = ParsePageQueueStatusError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            other => Err(ParsePageQueueStatusError(other.to_string())),
        }
    }
}

crate::impl_display_via_as_str!(PageQueueStatus);

/// A page item in the analysis queue.
/// Tracks pages to be analyzed, enabling resumability and concurrent processing.
#[derive(Debug, Clone, Serialize)]
pub struct PageQueueItem {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: Depth,
    pub status: PageQueueStatus,
    pub retry_count: RetryCount,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // ── HTML cache (populated during discovery) ──────────────────────
    /// Raw HTML body cached from the discovery fetch. `None` for
    /// pre-migration queue items or after analysis clears it.
    pub cached_html: Option<String>,
    pub http_status: Option<u16>,
    pub cached_load_time_ms: Option<f64>,
    pub final_url: Option<String>,
}

impl PageQueueItem {
    /// Create a new pending page queue item.
    ///
    /// `depth` is bounds-clamped via `Depth::new`; an out-of-range integer
    /// falls back to `Depth::root()` rather than panicking, since the
    /// caller is typically a SQL row decoder where validation has already
    /// happened upstream.
    pub fn new(job_id: &str, url: &str, depth: i64) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_id: job_id.to_string(),
            url: url.to_string(),
            depth: Depth::new(depth).unwrap_or_else(|_| Depth::root()),
            status: PageQueueStatus::Pending,
            retry_count: RetryCount::zero(),
            error_message: None,
            created_at: now,
            updated_at: now,
            cached_html: None,
            http_status: None,
            cached_load_time_ms: None,
            final_url: None,
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
        self.retry_count = self.retry_count.increment();
        self.updated_at = Utc::now();
    }

    /// Check if the item can be retried.
    pub fn can_retry(&self, max_retries: i64) -> bool {
        self.retry_count.can_retry(max_retries) && self.status == PageQueueStatus::Failed
    }
}

/// New page queue item for insertion.
#[derive(Debug, Clone)]
pub struct NewPageQueueItem {
    pub job_id: String,
    pub url: String,
    pub depth: Depth,
    // ── HTML cache from discovery ─────────────────────────────────────
    pub cached_html: Option<String>,
    pub http_status: Option<u16>,
    pub cached_load_time_ms: Option<f64>,
    pub final_url: Option<String>,
}

impl NewPageQueueItem {
    pub fn new(job_id: &str, url: &str, depth: Depth) -> Self {
        Self {
            job_id: job_id.to_string(),
            url: url.to_string(),
            depth,
            cached_html: None,
            http_status: None,
            cached_load_time_ms: None,
            final_url: None,
        }
    }

    /// Create from a discovery result, caching the fetched HTML.
    pub fn from_discovered(
        job_id: &str,
        page: &crate::service::discovery::DiscoveredPage,
        depth: Depth,
    ) -> Self {
        Self {
            job_id: job_id.to_string(),
            url: page.url.clone(),
            depth,
            cached_html: if page.html.is_empty() { None } else { Some(page.html.clone()) },
            http_status: if page.status_code == 0 { None } else { Some(page.status_code) },
            cached_load_time_ms: if page.load_time_ms == 0.0 { None } else { Some(page.load_time_ms) },
            final_url: Some(page.final_url.clone()),
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
            depth: Depth::root(),
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

    #[test]
    fn mobile_friendly_heuristic_requires_viewport() {
        let page = make_page(|p| p.has_viewport = false);
        assert!(!page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn mobile_friendly_heuristic_requires_load_time() {
        let page = make_page(|p| p.load_time_ms = None);
        assert!(!page.is_mobile_friendly_heuristic());
    }

    // ── PageQueueStatus ──────────────────────────────────────────────────────

    #[test]
    fn page_queue_status_round_trips_through_str() {
        use std::str::FromStr;
        for s in [
            PageQueueStatus::Pending,
            PageQueueStatus::Processing,
            PageQueueStatus::Completed,
            PageQueueStatus::Failed,
        ] {
            assert_eq!(PageQueueStatus::from_str(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn page_queue_status_from_str_is_case_insensitive() {
        use std::str::FromStr;
        assert_eq!(
            PageQueueStatus::from_str("PENDING").unwrap(),
            PageQueueStatus::Pending
        );
        assert_eq!(
            PageQueueStatus::from_str("Completed").unwrap(),
            PageQueueStatus::Completed
        );
    }

    #[test]
    fn page_queue_status_from_str_rejects_unknown() {
        use std::str::FromStr;
        assert!(PageQueueStatus::from_str("nope").is_err());
    }

    #[test]
    fn page_queue_status_error_carries_offending_input_in_display() {
        use std::str::FromStr;
        let err = PageQueueStatus::from_str("nonsense").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("nonsense"));
        assert!(msg.contains("invalid page queue status"));
    }

    #[test]
    fn page_queue_status_error_normalises_to_lowercase_in_payload() {
        // Pinning the to_lowercase() in from_str — the error carries
        // the lowercased form, not the original case.
        use std::str::FromStr;
        let err = PageQueueStatus::from_str("WHATEVER").unwrap_err();
        assert_eq!(err.0, "whatever");
    }

    // ── PageQueueItem ────────────────────────────────────────────────────────

    #[test]
    fn page_queue_item_new_starts_pending_with_zero_retries() {
        let item = PageQueueItem::new("job-1", "https://example.com", 2);
        assert_eq!(item.status, PageQueueStatus::Pending);
        assert_eq!(item.retry_count, RetryCount::zero());
        assert!(item.error_message.is_none());
        assert_eq!(item.depth.as_i64(), 2);
        assert_eq!(item.job_id, "job-1");
        assert!(!item.id.is_empty());
    }

    #[test]
    fn page_queue_item_with_id_uses_supplied_id() {
        let item = PageQueueItem::with_id("custom-id", "job-1", "https://x.test", 0);
        assert_eq!(item.id, "custom-id");
    }

    #[test]
    fn mark_processing_transitions_status_only() {
        let mut item = PageQueueItem::new("j", "u", 0);
        let initial_retries = item.retry_count;
        item.mark_processing();
        assert_eq!(item.status, PageQueueStatus::Processing);
        assert_eq!(item.retry_count, initial_retries);
        assert!(item.error_message.is_none());
    }

    #[test]
    fn mark_completed_transitions_status_only() {
        let mut item = PageQueueItem::new("j", "u", 0);
        item.mark_processing();
        item.mark_completed();
        assert_eq!(item.status, PageQueueStatus::Completed);
        assert!(item.error_message.is_none());
    }

    #[test]
    fn mark_failed_records_error_and_increments_retry() {
        let mut item = PageQueueItem::new("j", "u", 0);
        item.mark_failed("dns lookup failed");
        assert_eq!(item.status, PageQueueStatus::Failed);
        assert_eq!(item.retry_count.as_i64(), 1);
        assert_eq!(item.error_message.as_deref(), Some("dns lookup failed"));
    }

    #[test]
    fn repeated_mark_failed_increments_retry_each_time() {
        let mut item = PageQueueItem::new("j", "u", 0);
        item.mark_failed("e1");
        item.mark_failed("e2");
        item.mark_failed("e3");
        assert_eq!(item.retry_count.as_i64(), 3);
        assert_eq!(item.error_message.as_deref(), Some("e3"));
    }

    #[test]
    fn can_retry_only_when_failed_and_under_limit() {
        let mut item = PageQueueItem::new("j", "u", 0);
        // Pending: cannot retry
        assert!(!item.can_retry(3));

        // Failed once: can retry up to limit 3
        item.mark_failed("e");
        assert!(item.can_retry(3));

        // Failed three times: at the limit
        item.mark_failed("e");
        item.mark_failed("e");
        assert!(!item.can_retry(3));
    }

    #[test]
    fn can_retry_returns_false_for_completed_item() {
        let mut item = PageQueueItem::new("j", "u", 0);
        item.mark_completed();
        assert!(!item.can_retry(99));
    }

    #[test]
    fn new_page_queue_item_carries_input_fields() {
        let item =
            NewPageQueueItem::new("job-7", "https://example.com/p", Depth::new(4).unwrap());
        assert_eq!(item.job_id, "job-7");
        assert_eq!(item.url, "https://example.com/p");
        assert_eq!(item.depth.as_i64(), 4);
    }
}