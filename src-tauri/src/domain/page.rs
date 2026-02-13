use chrono::{DateTime, Utc};
use serde::Serialize;

/// A crawled page with SEO data.
/// Maps to the `pages` table in the new schema.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: i64,
    pub status_code: Option<i64>,
    pub content_type: Option<String>,

    // Core SEO fields
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub canonical_url: Option<String>,
    pub robots_meta: Option<String>,

    // Content metrics
    pub word_count: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub response_size_bytes: Option<i64>,

    pub crawled_at: DateTime<Utc>,
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
