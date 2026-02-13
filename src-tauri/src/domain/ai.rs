use chrono::{DateTime, Utc};
use serde::Serialize;

/// AI-generated insights for a job.
#[derive(Debug, Clone, Serialize)]
pub struct AiInsight {
    pub id: i64,
    pub job_id: String,
    pub summary: Option<String>,
    pub recommendations: Option<String>,
    pub raw_response: Option<String>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
