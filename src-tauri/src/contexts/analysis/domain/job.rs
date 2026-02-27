use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSettings {
    pub max_pages: i64,
    pub include_subdomains: bool,
    pub check_images: bool,
    pub mobile_analysis: bool,
    pub lighthouse_analysis: bool,
    pub delay_between_requests: i64,
}

impl Default for JobSettings {
    fn default() -> Self {
        Self {
            max_pages: 100,
            include_subdomains: true,
            check_images: true,
            mobile_analysis: false,
            lighthouse_analysis: false,
            delay_between_requests: 500,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobSummary {
    pub total_pages: i64,
    pub pages_crawled: i64,
    pub total_issues: i64,
    pub critical_issues: i64,
    pub warning_issues: i64,
    pub info_issues: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub settings: JobSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub summary: JobSummary,
    pub progress: f64,
    pub error_message: Option<String>,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
}

impl Job {
    /// Create a new job with default settings.
    pub fn new(url: String, settings: JobSettings) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            status: JobStatus::Pending,
            settings,
            created_at: now,
            updated_at: now,
            completed_at: None,
            summary: JobSummary::default(),
            progress: 0.0,
            error_message: None,
            sitemap_found: false,
            robots_txt_found: false,
        }
    }

    /// Create a new job with custom settings.
    pub fn with_settings(url: String, settings: JobSettings) -> Self {
        let mut job = Self::new(url, settings.clone());
        job.settings = settings;
        job
    }

    pub fn calculate_seo_score(&self) -> i64 {
        let total = self.summary.total_issues;
        if total == 0 {
            return 100;
        }

        let deductions = (self.summary.critical_issues * 10)
            + (self.summary.warning_issues * 5)
            + (total - self.summary.critical_issues - self.summary.warning_issues);

        (100 - deductions).clamp(0, 100)
    }
}

/// Lightweight job info for listing (without full settings/summary).
#[derive(Debug, Clone, Serialize)]
pub struct JobInfo {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub progress: f64,
    pub total_pages: i64,
    pub total_issues: i64,
    pub created_at: DateTime<Utc>,
    pub max_pages: i64,
    pub lighthouse_analysis: bool,
}

/// Status of an SEO analysis job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Discovery,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ParseJobStatusError(String);

impl std::fmt::Display for ParseJobStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid job status: '{}'", self.0)
    }
}

impl std::error::Error for ParseJobStatusError {}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Discovery => "discovery",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Discovery | Self::Processing)
    }
}

impl std::str::FromStr for JobStatus {
    type Err = ParseJobStatusError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" | "queued" => Ok(Self::Pending),
            "discovery" | "discovering" => Ok(Self::Discovery),
            "processing" | "analyzing" | "running" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(ParseJobStatusError(other.to_string())),
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CompleteJobResult {
    pub job: Job,
    pub pages: Vec<super::Page>,
    pub issues: Vec<super::Issue>,
    pub links: Vec<super::Link>,
    pub lighthouse: Vec<super::LighthouseData>,
    pub headings: Vec<super::Heading>,
    pub images: Vec<super::Image>,
    pub ai_insights: Option<crate::contexts::ai::AiInsight>,
    /// Extracted data from custom extractors (keyed by page_id)
    pub extracted_data: std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>,
}
