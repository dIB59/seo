use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Status of an SEO analysis job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::str::FromStr for JobStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" | "queued" => Ok(Self::Pending),
            "running" | "processing" | "discovering" | "analyzing" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Job settings for crawl configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSettings {
    pub max_pages: i64,
    pub include_external_links: bool,
    pub check_images: bool,
    pub mobile_analysis: bool,
    pub lighthouse_analysis: bool,
    pub delay_between_requests: i64,
}

impl Default for JobSettings {
    fn default() -> Self {
        Self {
            max_pages: 100,
            include_external_links: false,
            check_images: true,
            mobile_analysis: false,
            lighthouse_analysis: false,
            delay_between_requests: 500,
        }
    }
}

/// Summary statistics for a job (denormalized for fast dashboard access).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobSummary {
    pub total_pages: i64,
    pub pages_crawled: i64,
    pub total_issues: i64,
    pub critical_issues: i64,
    pub warning_issues: i64,
    pub info_issues: i64,
}

/// Consolidated job entity - combines job metadata, settings, and summary.
/// Maps to the `jobs` table in the new schema.
#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,

    // Settings (embedded)
    pub settings: JobSettings,

    // Summary stats (denormalized)
    pub summary: JobSummary,

    // Progress tracking
    pub progress: f64,
    pub current_stage: Option<String>,
    pub error_message: Option<String>,
}

impl Job {
    /// Create a new job with default settings.
    pub fn new(url: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            status: JobStatus::Pending,
            created_at: now,
            updated_at: now,
            completed_at: None,
            settings: JobSettings::default(),
            summary: JobSummary::default(),
            progress: 0.0,
            current_stage: None,
            error_message: None,
        }
    }

    /// Create a new job with custom settings.
    pub fn with_settings(url: String, settings: JobSettings) -> Self {
        let mut job = Self::new(url);
        job.settings = settings;
        job
    }

    /// Calculate SEO score based on issue counts.
    pub fn calculate_seo_score(&self) -> i64 {
        let total = self.summary.total_issues;
        let critical = self.summary.critical_issues;
        let warning = self.summary.warning_issues;

        if total == 0 {
            return 100;
        }

        let deductions = (critical * 10) + (warning * 5) + (total - critical - warning);
        let score = 100 - deductions;

        score.clamp(0, 100)
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

/// Complete job result with all related data.
#[derive(Debug, Clone, Serialize)]
pub struct CompleteJobResult {
    pub job: Job,
    pub pages: Vec<crate::domain::page::Page>,
    pub issues: Vec<crate::domain::issue::Issue>,
    pub links: Vec<crate::domain::link::Link>,
    pub lighthouse: Vec<crate::domain::lighthouse::LighthouseData>,
    pub headings: Vec<crate::domain::Heading>,
    pub images: Vec<crate::domain::Image>,
    pub ai_insights: Option<crate::domain::ai::AiInsight>,
}
