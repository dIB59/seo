//! Typestate wrapper for [`Job`] lifecycle transitions.
//!
//! `Job` is a row-shaped struct with a runtime `JobStatus` field. That's
//! convenient for SQL but invites bugs: nothing stops a caller from
//! marking an already-completed job as `Processing`, or reading
//! `error_message` on a `Pending` job, etc.
//!
//! `JobState<S>` wraps a `Job` with a phantom marker `S` so the compiler
//! knows which lifecycle stage it's in. Transitions consume `self` and
//! return a `JobState<NewState>`, making "mark a Completed job as
//! Processing" a compile error rather than a runtime check.
//!
//! ## Migration plan
//!
//! - **Step 1 (this commit)**: types live alongside the existing `Job`.
//!   Repositories continue to load `Job`; callers that want lifecycle
//!   safety convert via `AnyJob::from(job)` and match on the variant.
//! - **Step 2 (later)**: repository methods that already imply a state
//!   (`get_pending` → `Vec<JobState<Pending>>`) get typed return values.
//! - **Step 3 (later)**: every loader returns `AnyJob` and call sites
//!   exhaustively match.

use std::marker::PhantomData;

use super::{Job, JobStatus};

// ── State markers ────────────────────────────────────────────────────────────

/// A job that has been created but not started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending;

/// A job that is in the discovery phase (sitemap/robots.txt fetch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Discovery;

/// A job that is actively analysing pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Processing;

/// A job that finished successfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Completed;

/// A job that failed with an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Failed;

/// A job that was cancelled by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

// ── JobState wrapper ─────────────────────────────────────────────────────────

/// A `Job` whose lifecycle stage is tracked at the type level.
///
/// Invariants enforced by construction:
/// - The wrapped `Job`'s `status` field always matches the marker `S`.
/// - You cannot construct a `JobState<S>` for a state the underlying
///   `Job` is not actually in (the `From<Job>` path goes through
///   `AnyJob`, which matches on the runtime `JobStatus`).
#[derive(Debug, Clone)]
pub struct JobState<S> {
    job: Job,
    _state: PhantomData<S>,
}

impl<S> JobState<S> {
    /// Borrow the underlying `Job`. Read-only access is safe in any state.
    pub fn job(&self) -> &Job {
        &self.job
    }

    /// Consume the typestate wrapper and return the inner `Job`.
    pub fn into_inner(self) -> Job {
        self.job
    }
}

// ── Lifecycle transitions ────────────────────────────────────────────────────
//
// Each `impl<State>` block defines the methods available in that state.
// Calling `start()` on a `JobState<Completed>` is a compile error because
// the method only exists on `JobState<Pending>`.

impl JobState<Pending> {
    /// Move from Pending → Discovery.
    pub fn start_discovery(mut self) -> JobState<Discovery> {
        self.job.status = JobStatus::Discovery;
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }

    /// Cancel a job before it has started.
    pub fn cancel(mut self) -> JobState<Cancelled> {
        self.job.status = JobStatus::Cancelled;
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }
}

impl JobState<Discovery> {
    /// Move from Discovery → Processing once the URL list is in.
    pub fn start_processing(mut self) -> JobState<Processing> {
        self.job.status = JobStatus::Processing;
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }

    /// Cancel during discovery.
    pub fn cancel(mut self) -> JobState<Cancelled> {
        self.job.status = JobStatus::Cancelled;
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }

    /// Discovery failed (e.g. seed URL unreachable).
    pub fn fail(mut self, message: impl Into<String>) -> JobState<Failed> {
        self.job.status = JobStatus::Failed;
        self.job.error_message = Some(message.into());
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }
}

impl JobState<Processing> {
    /// Mark the job as completed.
    pub fn complete(mut self) -> JobState<Completed> {
        self.job.status = JobStatus::Completed;
        self.job.completed_at = Some(chrono::Utc::now());
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }

    /// Mark the job as failed mid-processing.
    pub fn fail(mut self, message: impl Into<String>) -> JobState<Failed> {
        self.job.status = JobStatus::Failed;
        self.job.error_message = Some(message.into());
        self.job.completed_at = Some(chrono::Utc::now());
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }

    /// Cancel during processing.
    pub fn cancel(mut self) -> JobState<Cancelled> {
        self.job.status = JobStatus::Cancelled;
        self.job.completed_at = Some(chrono::Utc::now());
        JobState {
            job: self.job,
            _state: PhantomData,
        }
    }

    /// Update progress as a percentage [0, 100].
    pub fn update_progress(&mut self, pct: f64) {
        self.job.progress = pct.clamp(0.0, 100.0);
    }
}

impl JobState<Completed> {
    /// Final SEO score for a completed job.
    pub fn seo_score(&self) -> i64 {
        self.job.calculate_seo_score()
    }
}

impl JobState<Failed> {
    /// Error message that caused the failure.
    pub fn error_message(&self) -> Option<&str> {
        self.job.error_message.as_deref()
    }
}

// ── Runtime dispatch enum ────────────────────────────────────────────────────

/// A job loaded from storage whose runtime state determines which
/// `JobState<S>` it maps to. Pattern-match on this to handle every
/// lifecycle stage exhaustively.
#[derive(Debug, Clone)]
pub enum AnyJob {
    Pending(JobState<Pending>),
    Discovery(JobState<Discovery>),
    Processing(JobState<Processing>),
    Completed(JobState<Completed>),
    Failed(JobState<Failed>),
    Cancelled(JobState<Cancelled>),
}

impl AnyJob {
    /// Borrow the underlying `Job` regardless of state.
    pub fn job(&self) -> &Job {
        match self {
            Self::Pending(s) => s.job(),
            Self::Discovery(s) => s.job(),
            Self::Processing(s) => s.job(),
            Self::Completed(s) => s.job(),
            Self::Failed(s) => s.job(),
            Self::Cancelled(s) => s.job(),
        }
    }

    /// Consume into the inner `Job`.
    pub fn into_inner(self) -> Job {
        match self {
            Self::Pending(s) => s.into_inner(),
            Self::Discovery(s) => s.into_inner(),
            Self::Processing(s) => s.into_inner(),
            Self::Completed(s) => s.into_inner(),
            Self::Failed(s) => s.into_inner(),
            Self::Cancelled(s) => s.into_inner(),
        }
    }

    /// Whether the job is in a terminal state (Completed, Failed, or
    /// Cancelled). Mirrors `JobStatus::is_terminal` but in the typestate
    /// world — useful when service code wants a quick yes/no without
    /// matching every variant.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed(_) | Self::Failed(_) | Self::Cancelled(_)
        )
    }

    /// Whether the job is actively running (Discovery or Processing).
    /// `Pending` is intentionally excluded — the job is queued but not
    /// yet doing work, matching `JobStatus::is_active`.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Discovery(_) | Self::Processing(_))
    }

    /// Lifecycle stage as a stable string identifier. Mirrors
    /// `JobStatus::as_str` so the typestate variant always agrees
    /// with the underlying status.
    pub fn stage_name(&self) -> &'static str {
        match self {
            Self::Pending(_) => "pending",
            Self::Discovery(_) => "discovery",
            Self::Processing(_) => "processing",
            Self::Completed(_) => "completed",
            Self::Failed(_) => "failed",
            Self::Cancelled(_) => "cancelled",
        }
    }

    // ── State extraction ────────────────────────────────────────────────

    /// Try to extract a `Pending` state, returning `None` if the job is
    /// in any other state. Useful in tests and at API entry points where
    /// the caller knows what state to expect.
    pub fn try_into_pending(self) -> Option<JobState<Pending>> {
        match self {
            Self::Pending(s) => Some(s),
            _ => None,
        }
    }

    pub fn try_into_discovery(self) -> Option<JobState<Discovery>> {
        match self {
            Self::Discovery(s) => Some(s),
            _ => None,
        }
    }

    pub fn try_into_processing(self) -> Option<JobState<Processing>> {
        match self {
            Self::Processing(s) => Some(s),
            _ => None,
        }
    }

    pub fn try_into_completed(self) -> Option<JobState<Completed>> {
        match self {
            Self::Completed(s) => Some(s),
            _ => None,
        }
    }

    pub fn try_into_failed(self) -> Option<JobState<Failed>> {
        match self {
            Self::Failed(s) => Some(s),
            _ => None,
        }
    }

    pub fn try_into_cancelled(self) -> Option<JobState<Cancelled>> {
        match self {
            Self::Cancelled(s) => Some(s),
            _ => None,
        }
    }
}

impl From<Job> for AnyJob {
    fn from(job: Job) -> Self {
        match job.status {
            JobStatus::Pending => Self::Pending(JobState {
                job,
                _state: PhantomData,
            }),
            JobStatus::Discovery => Self::Discovery(JobState {
                job,
                _state: PhantomData,
            }),
            JobStatus::Processing => Self::Processing(JobState {
                job,
                _state: PhantomData,
            }),
            JobStatus::Completed => Self::Completed(JobState {
                job,
                _state: PhantomData,
            }),
            JobStatus::Failed => Self::Failed(JobState {
                job,
                _state: PhantomData,
            }),
            JobStatus::Cancelled => Self::Cancelled(JobState {
                job,
                _state: PhantomData,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::analysis::JobSettings;

    fn fresh_job() -> Job {
        Job::new("https://example.com".to_string(), JobSettings::default())
    }

    #[test]
    fn pending_can_transition_to_discovery() {
        let job = fresh_job();
        let pending: JobState<Pending> = AnyJob::from(job).try_into_pending().unwrap();
        let discovery = pending.start_discovery();
        assert_eq!(discovery.job().status, JobStatus::Discovery);
    }

    #[test]
    fn discovery_can_transition_to_processing() {
        let job = fresh_job();
        let pending: JobState<Pending> = AnyJob::from(job).try_into_pending().unwrap();
        let processing = pending.start_discovery().start_processing();
        assert_eq!(processing.job().status, JobStatus::Processing);
    }

    #[test]
    fn processing_can_complete() {
        let job = fresh_job();
        let pending: JobState<Pending> = AnyJob::from(job).try_into_pending().unwrap();
        let completed = pending
            .start_discovery()
            .start_processing()
            .complete();
        assert_eq!(completed.job().status, JobStatus::Completed);
        assert!(completed.job().completed_at.is_some());
        assert_eq!(completed.seo_score(), 100); // no issues
    }

    #[test]
    fn processing_can_fail_with_message() {
        let job = fresh_job();
        let pending: JobState<Pending> = AnyJob::from(job).try_into_pending().unwrap();
        let failed = pending
            .start_discovery()
            .start_processing()
            .fail("network timeout");
        assert_eq!(failed.job().status, JobStatus::Failed);
        assert_eq!(failed.error_message(), Some("network timeout"));
        assert!(failed.job().completed_at.is_some());
    }

    #[test]
    fn pending_can_be_cancelled() {
        let job = fresh_job();
        let pending: JobState<Pending> = AnyJob::from(job).try_into_pending().unwrap();
        let cancelled = pending.cancel();
        assert_eq!(cancelled.job().status, JobStatus::Cancelled);
    }

    #[test]
    fn processing_progress_clamps_to_range() {
        let job = fresh_job();
        let pending: JobState<Pending> = AnyJob::from(job).try_into_pending().unwrap();
        let mut processing = pending.start_discovery().start_processing();
        processing.update_progress(150.0);
        assert_eq!(processing.job().progress, 100.0);
        processing.update_progress(-10.0);
        assert_eq!(processing.job().progress, 0.0);
        processing.update_progress(42.5);
        assert_eq!(processing.job().progress, 42.5);
    }

    #[test]
    fn any_job_dispatches_each_status_to_correct_variant() {
        let mut job = fresh_job();

        job.status = JobStatus::Pending;
        assert!(matches!(AnyJob::from(job.clone()), AnyJob::Pending(_)));

        job.status = JobStatus::Discovery;
        assert!(matches!(AnyJob::from(job.clone()), AnyJob::Discovery(_)));

        job.status = JobStatus::Processing;
        assert!(matches!(AnyJob::from(job.clone()), AnyJob::Processing(_)));

        job.status = JobStatus::Completed;
        assert!(matches!(AnyJob::from(job.clone()), AnyJob::Completed(_)));

        job.status = JobStatus::Failed;
        assert!(matches!(AnyJob::from(job.clone()), AnyJob::Failed(_)));

        job.status = JobStatus::Cancelled;
        assert!(matches!(AnyJob::from(job), AnyJob::Cancelled(_)));
    }

    #[test]
    fn any_job_round_trip_preserves_underlying_job() {
        let original = fresh_job();
        let original_id = original.id.clone();
        let any = AnyJob::from(original);
        let job = any.into_inner();
        assert_eq!(job.id, original_id);
    }

    #[test]
    fn full_lifecycle_pending_to_completed() {
        let job = fresh_job();
        let any = AnyJob::from(job);
        let result = match any {
            AnyJob::Pending(p) => Some(p.start_discovery().start_processing().complete()),
            _ => None,
        };
        assert!(result.is_some());
        assert_eq!(result.unwrap().job().status, JobStatus::Completed);
    }

    #[test]
    fn discovery_can_fail_during_seed_fetch() {
        let job = fresh_job();
        let any = AnyJob::from(job);
        let failed = match any {
            AnyJob::Pending(p) => p.start_discovery().fail("seed url 404"),
            _ => panic!("expected pending"),
        };
        assert_eq!(failed.error_message(), Some("seed url 404"));
    }

    #[test]
    fn any_job_is_terminal_returns_true_for_completed_failed_cancelled() {
        let mut job = fresh_job();
        job.status = JobStatus::Completed;
        assert!(AnyJob::from(job.clone()).is_terminal());

        job.status = JobStatus::Failed;
        assert!(AnyJob::from(job.clone()).is_terminal());

        job.status = JobStatus::Cancelled;
        assert!(AnyJob::from(job).is_terminal());
    }

    #[test]
    fn any_job_is_terminal_returns_false_for_in_flight_states() {
        let mut job = fresh_job();
        job.status = JobStatus::Pending;
        assert!(!AnyJob::from(job.clone()).is_terminal());

        job.status = JobStatus::Discovery;
        assert!(!AnyJob::from(job.clone()).is_terminal());

        job.status = JobStatus::Processing;
        assert!(!AnyJob::from(job).is_terminal());
    }

    #[test]
    fn any_job_is_active_only_for_discovery_and_processing() {
        let mut job = fresh_job();
        // Pending is queued, not yet active.
        job.status = JobStatus::Pending;
        assert!(!AnyJob::from(job.clone()).is_active());

        job.status = JobStatus::Discovery;
        assert!(AnyJob::from(job.clone()).is_active());

        job.status = JobStatus::Processing;
        assert!(AnyJob::from(job.clone()).is_active());

        // Terminal states are not active either.
        job.status = JobStatus::Completed;
        assert!(!AnyJob::from(job.clone()).is_active());
        job.status = JobStatus::Failed;
        assert!(!AnyJob::from(job.clone()).is_active());
        job.status = JobStatus::Cancelled;
        assert!(!AnyJob::from(job).is_active());
    }

    #[test]
    fn any_job_active_and_terminal_are_disjoint() {
        // No status should be both active and terminal — pinning the
        // invariant `JobStatus` itself enforces.
        for status in [
            JobStatus::Pending,
            JobStatus::Discovery,
            JobStatus::Processing,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ] {
            let mut job = fresh_job();
            job.status = status.clone();
            let any = AnyJob::from(job);
            assert!(
                !(any.is_active() && any.is_terminal()),
                "status {status:?} reported both active and terminal"
            );
        }
    }

    #[test]
    fn any_job_stage_name_matches_job_status_as_str() {
        // Pinning that the typestate variant always agrees with the
        // underlying JobStatus string form. Catches a future drift
        // where someone adds a state to one but not the other.
        for status in [
            JobStatus::Pending,
            JobStatus::Discovery,
            JobStatus::Processing,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ] {
            let mut job = fresh_job();
            job.status = status.clone();
            let any = AnyJob::from(job);
            assert_eq!(any.stage_name(), status.as_str());
        }
    }
}

