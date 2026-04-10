use super::ids::JobId;
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobSummary {
    total_pages: i64,
    pages_crawled: i64,
    total_issues: i64,
    critical_issues: i64,
    warning_issues: i64,
    info_issues: i64,
}

impl JobSummary {
    /// Construct a fully-populated JobSummary. Single entry point so
    /// any future invariant validation can be added in one place
    /// without a cascade through every construction site.
    pub fn new(
        total_pages: i64,
        pages_crawled: i64,
        total_issues: i64,
        critical_issues: i64,
        warning_issues: i64,
        info_issues: i64,
    ) -> Self {
        Self {
            total_pages,
            pages_crawled,
            total_issues,
            critical_issues,
            warning_issues,
            info_issues,
        }
    }

    pub fn total_pages(&self) -> i64 {
        self.total_pages
    }
    pub fn pages_crawled(&self) -> i64 {
        self.pages_crawled
    }
    pub fn total_issues(&self) -> i64 {
        self.total_issues
    }
    pub fn critical_issues(&self) -> i64 {
        self.critical_issues
    }
    pub fn warning_issues(&self) -> i64 {
        self.warning_issues
    }
    pub fn info_issues(&self) -> i64 {
        self.info_issues
    }

    /// Whether this job's analysis surfaced any issues at all.
    /// Equivalent to `total_issues > 0` but reads at the call site.
    pub fn has_issues(&self) -> bool {
        self.total_issues > 0
    }

    /// Whether this job has any critical issues — the most severe
    /// category. Useful for surfacing a "broken site" banner.
    pub fn has_critical_issues(&self) -> bool {
        self.critical_issues > 0
    }

    /// Whether the crawler made any progress yet. Distinct from
    /// `has_issues()` — a job with 0 pages crawled has no issues but
    /// is also not "complete".
    pub fn is_empty(&self) -> bool {
        self.pages_crawled == 0
    }

    /// Number of issues that are NOT critical or warning — i.e. info
    /// / suggestion severity. Computed from the breakdown so the
    /// caller doesn't have to do the arithmetic.
    pub fn info_or_below_issues(&self) -> i64 {
        // Pin: derive from total - (critical + warning), not from the
        // info_issues field directly. The info_issues field tracks
        // *info-tagged* issues specifically; the actual "everything
        // not critical/warning" bucket is the remainder. The two
        // happen to coincide today but the test below pins the
        // distinction.
        self.total_issues - self.critical_issues - self.warning_issues
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: JobId,
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
            id: JobId::generate(),
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

    pub fn calculate_seo_score(&self) -> i64 {
        // Use the JobSummary predicates so the call site reads as
        // English. The arithmetic is: critical → -10 each, warning →
        // -5 each, info or below → -1 each.
        if !self.summary.has_issues() {
            return 100;
        }

        let deductions = (self.summary.critical_issues() * 10)
            + (self.summary.warning_issues() * 5)
            + self.summary.info_or_below_issues();

        (100 - deductions).clamp(0, 100)
    }
}

#[cfg(test)]
mod tests {
    //! Characterization tests pinning the current behavior of `Job` and
    //! `JobStatus`. These exist so a future refactor that privatizes fields
    //! and introduces typestate transitions (see plan: Phase 5) can land
    //! under green without changing observable semantics.

    use super::*;

    fn make_summary(total: i64, critical: i64, warning: i64) -> JobSummary {
        JobSummary {
            total_issues: total,
            critical_issues: critical,
            warning_issues: warning,
            ..Default::default()
        }
    }

    #[test]
    fn new_initializes_to_pending_with_zero_progress() {
        let job = Job::new("https://example.com".to_string(), JobSettings::default());
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.progress, 0.0);
        assert!(job.error_message.is_none());
        assert!(!job.sitemap_found);
        assert!(!job.robots_txt_found);
        assert_eq!(job.url, "https://example.com");
        assert_eq!(job.summary.total_issues, 0);
    }

    #[test]
    fn new_generates_unique_ids() {
        let a = Job::new("https://a.test".into(), JobSettings::default());
        let b = Job::new("https://b.test".into(), JobSettings::default());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn seo_score_is_perfect_with_no_issues() {
        let mut job = Job::new("https://x.test".into(), JobSettings::default());
        job.summary = make_summary(0, 0, 0);
        assert_eq!(job.calculate_seo_score(), 100);
    }

    #[test]
    fn seo_score_deducts_ten_per_critical() {
        let mut job = Job::new("https://x.test".into(), JobSettings::default());
        // 2 critical, 0 warning, 2 total → deductions = 20 + 0 + 0 = 20
        job.summary = make_summary(2, 2, 0);
        assert_eq!(job.calculate_seo_score(), 80);
    }

    #[test]
    fn seo_score_deducts_five_per_warning() {
        let mut job = Job::new("https://x.test".into(), JobSettings::default());
        // 0 critical, 3 warning, 3 total → deductions = 0 + 15 + 0 = 15
        job.summary = make_summary(3, 0, 3);
        assert_eq!(job.calculate_seo_score(), 85);
    }

    #[test]
    fn seo_score_deducts_one_per_info_issue() {
        let mut job = Job::new("https://x.test".into(), JobSettings::default());
        // 5 total, 0 critical, 0 warning → 5 info issues, each -1
        job.summary = make_summary(5, 0, 0);
        assert_eq!(job.calculate_seo_score(), 95);
    }

    #[test]
    fn seo_score_clamps_to_zero_floor() {
        let mut job = Job::new("https://x.test".into(), JobSettings::default());
        // 100 critical → 1000 deductions, would be -900, clamps to 0
        job.summary = make_summary(100, 100, 0);
        assert_eq!(job.calculate_seo_score(), 0);
    }

    #[test]
    fn job_status_from_str_accepts_aliases() {
        use std::str::FromStr;
        assert_eq!(JobStatus::from_str("pending").unwrap(), JobStatus::Pending);
        assert_eq!(JobStatus::from_str("queued").unwrap(), JobStatus::Pending);
        assert_eq!(JobStatus::from_str("running").unwrap(), JobStatus::Processing);
        assert_eq!(JobStatus::from_str("analyzing").unwrap(), JobStatus::Processing);
        assert_eq!(JobStatus::from_str("error").unwrap(), JobStatus::Failed);
    }

    #[test]
    fn job_status_from_str_is_case_insensitive() {
        use std::str::FromStr;
        assert_eq!(JobStatus::from_str("PENDING").unwrap(), JobStatus::Pending);
        assert_eq!(JobStatus::from_str("Completed").unwrap(), JobStatus::Completed);
    }

    #[test]
    fn job_status_from_str_rejects_unknown() {
        use std::str::FromStr;
        assert!(JobStatus::from_str("nonsense").is_err());
    }

    #[test]
    fn job_status_terminal_and_active_are_disjoint() {
        for s in [
            JobStatus::Pending,
            JobStatus::Discovery,
            JobStatus::Processing,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ] {
            // No status is both active and terminal.
            assert!(!(s.is_active() && s.is_terminal()), "{s:?}");
        }
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
        assert!(JobStatus::Discovery.is_active());
        assert!(JobStatus::Processing.is_active());
        assert!(!JobStatus::Pending.is_active());
        assert!(!JobStatus::Pending.is_terminal());
    }

    #[test]
    fn job_status_round_trips_through_str() {
        use std::str::FromStr;
        for s in [
            JobStatus::Pending,
            JobStatus::Discovery,
            JobStatus::Processing,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ] {
            let parsed = JobStatus::from_str(s.as_str()).expect("known status");
            assert_eq!(parsed, s);
        }
    }

    // ── JobSummary predicates ────────────────────────────────────────────

    #[test]
    fn summary_has_issues_returns_false_for_default() {
        assert!(!JobSummary::default().has_issues());
    }

    #[test]
    fn summary_has_issues_returns_true_when_total_positive() {
        let s = make_summary(5, 1, 2);
        assert!(s.has_issues());
    }

    #[test]
    fn summary_has_critical_issues_returns_true_only_for_critical_count() {
        let mut s = make_summary(10, 0, 5);
        assert!(!s.has_critical_issues());
        s.critical_issues = 1;
        assert!(s.has_critical_issues());
    }

    #[test]
    fn summary_is_empty_returns_true_for_zero_pages_crawled() {
        let s = JobSummary {
            pages_crawled: 0,
            ..Default::default()
        };
        assert!(s.is_empty());
    }

    #[test]
    fn summary_is_empty_returns_false_when_any_page_crawled() {
        let s = JobSummary {
            pages_crawled: 1,
            ..Default::default()
        };
        assert!(!s.is_empty());
    }

    #[test]
    fn summary_info_or_below_subtracts_critical_and_warning_from_total() {
        // 10 total, 3 critical, 4 warning → 3 below threshold
        let s = make_summary(10, 3, 4);
        assert_eq!(s.info_or_below_issues(), 3);
    }

    #[test]
    fn summary_info_or_below_handles_all_critical() {
        // Every issue is critical → 0 below threshold
        let s = make_summary(5, 5, 0);
        assert_eq!(s.info_or_below_issues(), 0);
    }

    #[test]
    fn summary_info_or_below_handles_zero_total() {
        // Default summary → 0 below threshold (no underflow)
        assert_eq!(JobSummary::default().info_or_below_issues(), 0);
    }
}

/// Lightweight job info for listing (without full settings/summary).
///
/// Fields are private — construct via [`JobInfo::new`] and read via
/// the typed accessors. The single construction site lets us add
/// invariants (e.g. "progress must be in [0, 100]") in one place
/// without a cascade through every repository decoder.
#[derive(Debug, Clone, Serialize)]
pub struct JobInfo {
    id: JobId,
    url: String,
    status: JobStatus,
    progress: f64,
    total_pages: i64,
    total_issues: i64,
    created_at: DateTime<Utc>,
    max_pages: i64,
    lighthouse_analysis: bool,
}

impl JobInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: JobId,
        url: String,
        status: JobStatus,
        progress: f64,
        total_pages: i64,
        total_issues: i64,
        created_at: DateTime<Utc>,
        max_pages: i64,
        lighthouse_analysis: bool,
    ) -> Self {
        Self {
            id,
            url,
            status,
            progress,
            total_pages,
            total_issues,
            created_at,
            max_pages,
            lighthouse_analysis,
        }
    }

    pub fn id(&self) -> &JobId {
        &self.id
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn status(&self) -> &JobStatus {
        &self.status
    }
    pub fn progress(&self) -> f64 {
        self.progress
    }
    pub fn total_pages(&self) -> i64 {
        self.total_pages
    }
    pub fn total_issues(&self) -> i64 {
        self.total_issues
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn max_pages(&self) -> i64 {
        self.max_pages
    }
    pub fn lighthouse_analysis(&self) -> bool {
        self.lighthouse_analysis
    }
}

/// Project a full [`Job`] down to the lightweight [`JobInfo`] view used by
/// listing endpoints. Centralized so the in-memory mock repo and any future
/// projection share one definition of "what fields the listing carries".
impl From<&Job> for JobInfo {
    fn from(job: &Job) -> Self {
        Self::new(
            job.id.clone(),
            job.url.clone(),
            job.status.clone(),
            job.progress,
            job.summary.total_pages(),
            job.summary.total_issues(),
            job.created_at,
            job.settings.max_pages,
            job.settings.lighthouse_analysis,
        )
    }
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

crate::impl_display_via_as_str!(JobStatus);

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
