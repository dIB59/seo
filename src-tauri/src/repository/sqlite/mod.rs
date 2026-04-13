mod ai_repository;
mod extension_repository;
mod issue_repository;
pub(super) mod job_repository;
mod link_repository;
mod page_queue_repository;
mod page_repository;
mod report_repository;
mod results_repository;
mod settings_repository;
mod template_repository;

pub use ai_repository::AiRepository;
pub use extension_repository::SqliteExtensionRepository;
pub use issue_repository::{IssueCounts, IssueGroup, IssueRepository};
pub use job_repository::JobRepository;
pub use link_repository::{ExternalDomain, LinkCounts, LinkRepository};
pub use page_queue_repository::PageQueueRepository;
pub use page_repository::PageRepository;
pub use report_repository::SqliteReportPatternRepository;
pub use template_repository::ReportTemplateRepository;
pub use results_repository::ResultsRepository;
pub use settings_repository::SettingsRepository;

use chrono::{DateTime, Utc};

use crate::contexts::analysis::Depth;
use crate::contexts::{IssueSeverity, JobStatus, LinkType};

/// Decode an RFC-3339 timestamp column. Malformed values fall back to the
/// current time after a warning so a single bad row can't fail the whole
/// query — same drift-tolerance pattern used by the other decoders here.
/// Centralized so the four `parse_datetime` copies in the per-table
/// repositories don't drift.
pub fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|e| {
            tracing::warn!("decoder: invalid timestamp '{s}' ({e}); defaulting to now");
            Utc::now()
        })
}

/// Decode a raw `depth` integer column. Out-of-range values fall back to
/// `Depth::root()` after a warning so a corrupted row doesn't break the
/// page listing query — older app versions wrote depths without bounds
/// checks, and this is the centralized place to absorb that drift.
pub fn decode_depth(raw: i64) -> Depth {
    Depth::new(raw).unwrap_or_else(|e| {
        tracing::warn!("decoder: invalid page depth {raw} ({e}); defaulting to root");
        Depth::root()
    })
}

/// Map a `rows_affected` count from an UPDATE/DELETE into a typed
/// not-found error when zero rows matched. Centralizes the
/// "did you actually touch a row?" check used by every repository
/// that exposes update/delete by id — keeps the error shape and
/// "entity" tag consistent across the file.
pub fn require_affected(
    rows_affected: u64,
    entity: &'static str,
    id: &str,
) -> crate::repository::RepositoryResult<()> {
    if rows_affected == 0 {
        return Err(crate::repository::RepositoryError::not_found(entity, id));
    }
    Ok(())
}

/// Decode the JSON `extracted_data` column. Malformed JSON falls back to an
/// empty map so a single broken row doesn't fail the whole query.
pub fn decode_extracted_data(
    raw: &str,
) -> std::collections::HashMap<String, serde_json::Value> {
    serde_json::from_str(raw).unwrap_or_else(|e| {
        tracing::warn!("decoder: invalid extracted_data JSON ({e}); defaulting to empty");
        std::collections::HashMap::new()
    })
}

/// Project a sqlx anonymous lighthouse row into a [`LighthouseData`].
/// Implemented as a macro because each `sqlx::query!` invocation produces
/// a distinct anonymous row type, so a generic function can't be expressed
/// without a wrapper struct or trait. Both lighthouse decoder sites
/// (`page_repository::get_lighthouse_for_job` and
/// `results_repository::get_complete_result`) select the exact same
/// columns, so this macro pins that contract in one place.
/// Project a sqlx anonymous job row into a [`Job`]. Same macro pattern as
/// [`lighthouse_data_from_row!`] — every `sqlx::query!` call produces a
/// distinct anonymous row type, so the body has to be expanded inline at
/// each call site even though every Job decoder selects the same columns.
/// Three decoders use this: `job_repository::get_by_id`,
/// `job_repository::get_pending`, and `results_repository::get_job`.
macro_rules! job_from_row {
    ($row:expr) => {{
        let row = $row;
        $crate::contexts::analysis::Job {
            id: $crate::contexts::analysis::JobId::from(row.id),
            url: row.url,
            status: super::map_job_status(&row.status),
            created_at: super::parse_datetime(&row.created_at),
            updated_at: super::parse_datetime(&row.updated_at),
            completed_at: row.completed_at.as_deref().map(super::parse_datetime),
            settings: super::job_repository::decode_job_settings(
                row.max_pages,
                row.include_subdomains,
                row.lighthouse_analysis,
                row.rate_limit_ms,
            ),
            summary: super::job_repository::decode_job_summary(
                row.total_pages,
                row.pages_crawled,
                row.total_issues,
                row.critical_issues,
                row.warning_issues,
                row.info_issues,
            ),
            progress: row.progress,
            error_message: row.error_message,
            sitemap_found: row.sitemap_found,
            robots_txt_found: row.robots_txt_found,
        }
    }};
}
pub(super) use job_from_row;

/// Project a sqlx anonymous page row into a [`Page`]. Three call sites:
/// `page_repository::get_by_job_id`, `page_repository::get_by_id`, and
/// `results_repository::get_pages`. Same macro rationale as
/// `job_from_row!` — sqlx anonymous row types can't be unified by a
/// generic function. Requires `extracted_data` to be in scope at the
/// call site (it's the only field whose decode is shaped differently
/// at each site — sometimes done up-front into a local, sometimes inline).
macro_rules! page_from_row {
    ($row:expr, $extracted_data:expr) => {{
        let row = $row;
        $crate::contexts::analysis::Page {
            id: row.id,
            job_id: row.job_id,
            url: row.url,
            depth: super::decode_depth(row.depth),
            status_code: row.status_code,
            content_type: row.content_type,
            title: row.title,
            meta_description: row.meta_description,
            canonical_url: row.canonical_url,
            robots_meta: row.robots_meta,
            word_count: row.word_count,
            load_time_ms: row.load_time_ms,
            response_size_bytes: row.response_size_bytes,
            has_viewport: row.has_viewport != 0,
            has_structured_data: row.has_structured_data != 0,
            crawled_at: super::parse_datetime(row.crawled_at.as_str()),
            extracted_data: $extracted_data,
        }
    }};
}
pub(super) use page_from_row;

macro_rules! lighthouse_data_from_row {
    ($row:expr) => {
        $crate::contexts::analysis::LighthouseData {
            page_id: $row.page_id,
            performance_score: $row.performance_score,
            accessibility_score: $row.accessibility_score,
            best_practices_score: $row.best_practices_score,
            seo_score: $row.seo_score,
            first_contentful_paint_ms: $row.first_contentful_paint_ms,
            largest_contentful_paint_ms: $row.largest_contentful_paint_ms,
            total_blocking_time_ms: $row.total_blocking_time_ms,
            cumulative_layout_shift: $row.cumulative_layout_shift,
            speed_index: $row.speed_index,
            time_to_interactive_ms: $row.time_to_interactive_ms,
            raw_json: $row.raw_json,
        }
    };
}
pub(crate) use lighthouse_data_from_row;

// Decoder helpers for stringly-typed columns. Each one falls back to a
// "least surprising" default when the database row contains an unknown
// value (e.g. a value written by an older app version), but logs a
// warning so the drift is visible. Previously these silently coerced
// junk to the default, which masked schema bugs and migration mishaps.

pub fn map_job_status(s: &str) -> JobStatus {
    s.parse().unwrap_or_else(|e| {
        tracing::warn!("decoder: unknown job status '{s}' ({e}); defaulting to Pending");
        JobStatus::Pending
    })
}

pub fn map_severity(s: &str) -> IssueSeverity {
    s.parse().unwrap_or_else(|e| {
        tracing::warn!("decoder: unknown issue severity '{s}' ({e}); defaulting to Info");
        IssueSeverity::Info
    })
}

pub fn map_link_type(s: &str) -> LinkType {
    s.parse().unwrap_or_else(|e| {
        tracing::warn!("decoder: unknown link type '{s}' ({e}); defaulting to Internal");
        LinkType::Internal
    })
}
