//! Unified results repository for the redesigned schema.
//!
//! This provides a high-level API to fetch complete job results with
//! all related data in optimized queries.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::domain::models_v2::{
    AiInsight, CompleteJobResult, Issue, Job, JobSettings, JobSummary,
    LighthouseData, Link, Page,
};
use super::{map_job_status_v2, map_link_type, map_severity};

pub struct ResultsRepositoryV2 {
    pool: SqlitePool,
}

impl ResultsRepositoryV2 {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get complete job result with all related data.
    /// This is the main query for displaying analysis results.
    ///
    /// Performance: With the new schema, this uses direct FK lookups:
    /// - 1 query for job
    /// - 1 query for pages (WHERE job_id = ?)
    /// - 1 query for issues (WHERE job_id = ?)
    /// - 1 query for links (WHERE job_id = ?)
    /// - 1 query for lighthouse (JOIN on pages)
    /// - 1 query for AI insights
    ///
    /// Total: 6 simple queries vs the old 5+ queries with expensive JOINs.
    pub async fn get_complete_result(&self, job_id: &str) -> Result<CompleteJobResult> {
        let query_start = std::time::Instant::now();

        // 1. Get job (includes settings and summary)
        let job = self.get_job(job_id).await?;

        // 2. Get pages (direct FK lookup - FAST!)
        let pages = self.get_pages(job_id).await?;
        log::debug!("Fetched {} pages in {:?}", pages.len(), query_start.elapsed());

        // 3. Get issues (direct FK lookup - FAST!)
        let issues = self.get_issues(job_id).await?;
        log::debug!("Fetched {} issues", issues.len());

        // 4. Get links (direct FK lookup - FAST!)
        let links = self.get_links(job_id).await?;
        log::debug!("Fetched {} links", links.len());

        // 5. Get lighthouse data
        let lighthouse = self.get_lighthouse(job_id).await?;
        log::debug!("Fetched {} lighthouse records", lighthouse.len());

        // 6. Get AI insights (optional)
        let ai_insights = self.get_ai_insights(job_id).await.ok();

        let total_time = query_start.elapsed();
        log::info!(
            "Loaded complete result for job {} with {} pages, {} issues, {} links in {:?}",
            job_id,
            pages.len(),
            issues.len(),
            links.len(),
            total_time
        );

        Ok(CompleteJobResult {
            job,
            pages,
            issues,
            links,
            lighthouse,
            ai_insights,
        })
    }

    async fn get_job(&self, job_id: &str) -> Result<Job> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent,
                total_pages, pages_crawled, total_issues, 
                critical_issues, warning_issues, info_issues,
                progress, current_stage, error_message
            FROM jobs
            WHERE id = ?
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch job")?;

        Ok(Job {
            id: row.get("id"),
            url: row.get("url"),
            status: map_job_status_v2(row.get::<&str, _>("status")),
            created_at: parse_datetime(row.get("created_at")),
            updated_at: parse_datetime(row.get("updated_at")),
            completed_at: row.get::<Option<&str>, _>("completed_at").map(parse_datetime),
            settings: JobSettings {
                max_pages: row.get("max_pages"),
                max_depth: row.get("max_depth"),
                respect_robots_txt: row.get::<i64, _>("respect_robots_txt") != 0,
                include_subdomains: row.get::<i64, _>("include_subdomains") != 0,
                rate_limit_ms: row.get("rate_limit_ms"),
                user_agent: row.get("user_agent"),
            },
            summary: JobSummary {
                total_pages: row.get("total_pages"),
                pages_crawled: row.get("pages_crawled"),
                total_issues: row.get("total_issues"),
                critical_issues: row.get("critical_issues"),
                warning_issues: row.get("warning_issues"),
                info_issues: row.get("info_issues"),
            },
            progress: row.get("progress"),
            current_stage: row.get("current_stage"),
            error_message: row.get("error_message"),
        })
    }

    async fn get_pages(&self, job_id: &str) -> Result<Vec<Page>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes, crawled_at
            FROM pages
            WHERE job_id = ?
            ORDER BY depth ASC, url ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pages")?;

        Ok(rows
            .into_iter()
            .map(|row| Page {
                id: row.get("id"),
                job_id: row.get("job_id"),
                url: row.get("url"),
                depth: row.get("depth"),
                status_code: row.get("status_code"),
                content_type: row.get("content_type"),
                title: row.get("title"),
                meta_description: row.get("meta_description"),
                canonical_url: row.get("canonical_url"),
                robots_meta: row.get("robots_meta"),
                word_count: row.get("word_count"),
                load_time_ms: row.get("load_time_ms"),
                response_size_bytes: row.get("response_size_bytes"),
                crawled_at: parse_datetime(row.get("crawled_at")),
            })
            .collect())
    }

    async fn get_issues(&self, job_id: &str) -> Result<Vec<Issue>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, job_id, page_id, type, severity, message, details, created_at
            FROM issues
            WHERE job_id = ?
            ORDER BY 
                CASE severity 
                    WHEN 'critical' THEN 1 
                    WHEN 'warning' THEN 2 
                    ELSE 3 
                END,
                type ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch issues")?;

        Ok(rows
            .into_iter()
            .map(|row| Issue {
                id: row.get("id"),
                job_id: row.get("job_id"),
                page_id: row.get("page_id"),
                issue_type: row.get("type"),
                severity: map_severity(row.get("severity")),
                message: row.get("message"),
                details: row.get("details"),
                created_at: parse_datetime(row.get("created_at")),
            })
            .collect())
    }

    async fn get_links(&self, job_id: &str) -> Result<Vec<Link>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, job_id, source_page_id, target_page_id, target_url,
                link_text, link_type, is_followed, status_code
            FROM links
            WHERE job_id = ?
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch links")?;

        Ok(rows
            .into_iter()
            .map(|row| Link {
                id: row.get("id"),
                job_id: row.get("job_id"),
                source_page_id: row.get("source_page_id"),
                target_page_id: row.get("target_page_id"),
                target_url: row.get("target_url"),
                link_text: row.get("link_text"),
                link_type: map_link_type(row.get("link_type")),
                is_followed: row.get::<i64, _>("is_followed") != 0,
                status_code: row.get("status_code"),
            })
            .collect())
    }

    async fn get_lighthouse(&self, job_id: &str) -> Result<Vec<LighthouseData>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                pl.page_id, pl.performance_score, pl.accessibility_score,
                pl.best_practices_score, pl.seo_score,
                pl.first_contentful_paint_ms, pl.largest_contentful_paint_ms,
                pl.total_blocking_time_ms, pl.cumulative_layout_shift,
                pl.speed_index, pl.time_to_interactive_ms, pl.raw_json
            FROM page_lighthouse pl
            JOIN pages p ON p.id = pl.page_id
            WHERE p.job_id = ?
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch lighthouse data")?;

        Ok(rows
            .into_iter()
            .map(|row| LighthouseData {
                page_id: row.get("page_id"),
                performance_score: row.get("performance_score"),
                accessibility_score: row.get("accessibility_score"),
                best_practices_score: row.get("best_practices_score"),
                seo_score: row.get("seo_score"),
                first_contentful_paint_ms: row.get("first_contentful_paint_ms"),
                largest_contentful_paint_ms: row.get("largest_contentful_paint_ms"),
                total_blocking_time_ms: row.get("total_blocking_time_ms"),
                cumulative_layout_shift: row.get("cumulative_layout_shift"),
                speed_index: row.get("speed_index"),
                time_to_interactive_ms: row.get("time_to_interactive_ms"),
                raw_json: row.get("raw_json"),
            })
            .collect())
    }

    async fn get_ai_insights(&self, job_id: &str) -> Result<AiInsight> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, job_id, summary, recommendations, raw_response,
                model, created_at, updated_at
            FROM ai_insights
            WHERE job_id = ?
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch AI insights")?;

        Ok(AiInsight {
            id: row.get("id"),
            job_id: row.get("job_id"),
            summary: row.get("summary"),
            recommendations: row.get("recommendations"),
            raw_response: row.get("raw_response"),
            model: row.get("model"),
            created_at: parse_datetime(row.get("created_at")),
            updated_at: parse_datetime(row.get("updated_at")),
        })
    }

    /// Save AI insights for a job.
    pub async fn save_ai_insights(
        &self,
        job_id: &str,
        summary: Option<&str>,
        recommendations: Option<&str>,
        raw_response: Option<&str>,
        model: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO ai_insights (job_id, summary, recommendations, raw_response, model, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(job_id) DO UPDATE SET
                summary = excluded.summary,
                recommendations = excluded.recommendations,
                raw_response = excluded.raw_response,
                model = excluded.model,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(job_id)
        .bind(summary)
        .bind(recommendations)
        .bind(raw_response)
        .bind(model)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .context("Failed to save AI insights")?;

        Ok(())
    }
}

/// Parse datetime string to UTC DateTime.
fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
