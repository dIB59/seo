use chrono::Utc;
use sqlx::SqlitePool;

use crate::contexts::{
    ai::AiInsight,
    analysis::{
        CompleteJobResult, Heading, Image, Issue, Job,
        LighthouseData, Link, Page,
    },
};

use super::{map_link_type, map_severity};
use crate::repository::{
    RepositoryError, RepositoryResult, ResultsRepository as ResultsRepositoryTrait,
};
use async_trait::async_trait;

pub struct ResultsRepository {
    pool: SqlitePool,
}

impl ResultsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ResultsRepositoryTrait for ResultsRepository {
    async fn get_complete_result(&self, job_id: &str) -> RepositoryResult<CompleteJobResult> {
        let query_start = std::time::Instant::now();

        // 1. Get job (includes settings and summary)
        let job = self.get_job(job_id).await?;

        // 2. Get pages (direct FK lookup - FAST!)
        let pages = self.get_pages(job_id).await?;
        tracing::debug!(
            "Fetched {} pages in {:?}",
            pages.len(),
            query_start.elapsed()
        );

        // 3. Get issues (direct FK lookup - FAST!)
        let issues = self.get_issues(job_id).await?;
        tracing::debug!("Fetched {} issues", issues.len());

        // 4. Get links (direct FK lookup - FAST!)
        let links = self.get_links(job_id).await?;
        tracing::debug!("Fetched {} links", links.len());

        // 5. Get lighthouse data
        let lighthouse = self.get_lighthouse(job_id).await?;
        tracing::debug!("Fetched {} lighthouse records", lighthouse.len());

        // 6. Get headings
        let headings = self.get_headings(job_id).await?;
        tracing::debug!("Fetched {} headings", headings.len());

        // 7. Get images
        let images = self.get_images(job_id).await?;
        tracing::debug!("Fetched {} images", images.len());

        // 8. Get AI insights (optional)
        let ai_insights = self.get_ai_insights(job_id).await.ok();

        let total_time = query_start.elapsed();
        tracing::info!(
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
            headings,
            images,
            ai_insights,
            extracted_data: std::collections::HashMap::new(),
        })
    }

    async fn get_job(&self, job_id: &str) -> RepositoryResult<Job> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis,
                total_pages, pages_crawled, total_issues, 
                critical_issues, warning_issues, info_issues,
                progress, error_message, sitemap_found, robots_txt_found
            FROM jobs
            WHERE id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::not_found("job", job_id),
            other => RepositoryError::from(other),
        })?;

        Ok(super::job_from_row!(row))
    }

    async fn get_pages(&self, job_id: &str) -> RepositoryResult<Vec<Page>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes,
                has_viewport, has_structured_data, crawled_at, extracted_data
            FROM pages
            WHERE job_id = ?
            ORDER BY depth ASC, url ASC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let extracted_data = super::decode_extracted_data(&row.extracted_data);
                super::page_from_row!(row, extracted_data)
            })
            .collect())
    }

    async fn get_issues(&self, job_id: &str) -> RepositoryResult<Vec<Issue>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id!", job_id, page_id, type as issue_type, severity, message, details, created_at
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
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| Issue {
                id: row.id,
                job_id: row.job_id,
                page_id: row.page_id,
                issue_type: row.issue_type,
                severity: map_severity(row.severity.as_str()),
                message: row.message,
                details: row.details,
                created_at: parse_datetime(row.created_at.as_str()),
            })
            .collect())
    }

    async fn get_links(&self, job_id: &str) -> RepositoryResult<Vec<Link>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id as "id!", job_id, source_page_id, target_url,
                link_text, link_type, status_code
            FROM links
            WHERE job_id = ?
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| Link {
                id: row.id.to_string(),
                job_id: row.job_id,
                source_page_id: row.source_page_id,
                target_url: row.target_url,
                link_text: row.link_text,
                link_type: map_link_type(row.link_type.as_str()),
                status_code: row.status_code,
            })
            .collect())
    }

    async fn get_lighthouse(&self, job_id: &str) -> RepositoryResult<Vec<LighthouseData>> {
        let rows = sqlx::query!(
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
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| super::lighthouse_data_from_row!(row))
            .collect())
    }

    async fn get_headings(&self, job_id: &str) -> RepositoryResult<Vec<Heading>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                ph.id as "id!", ph.page_id, ph.level, ph.text, ph.position
            FROM page_headings ph
            JOIN pages p ON p.id = ph.page_id
            WHERE p.job_id = ?
            ORDER BY ph.page_id, ph.position
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| Heading {
                id: row.id,
                page_id: row.page_id,
                level: row.level,
                text: row.text,
                position: row.position,
            })
            .collect())
    }

    async fn get_images(&self, job_id: &str) -> RepositoryResult<Vec<Image>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                pi.id as "id!", pi.page_id, pi.src, pi.alt, pi.width, pi.height,
                pi.loading, pi.is_decorative
            FROM page_images pi
            JOIN pages p ON p.id = pi.page_id
            WHERE p.job_id = ?
            ORDER BY pi.page_id, pi.id
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| Image {
                id: row.id,
                page_id: row.page_id,
                src: row.src,
                alt: row.alt,
                width: row.width,
                height: row.height,
                loading: row.loading,
                is_decorative: row.is_decorative != 0,
            })
            .collect())
    }

    async fn get_ai_insights(&self, job_id: &str) -> RepositoryResult<AiInsight> {
        let row = sqlx::query!(
            r#"
            SELECT
                id as "id!", job_id, summary, recommendations, raw_response,
                model, created_at, updated_at
            FROM ai_insights
            WHERE job_id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::not_found("ai_insights", job_id),
            other => RepositoryError::from(other),
        })?;

        Ok(AiInsight {
            id: row.id,
            job_id: row.job_id,
            summary: row.summary,
            recommendations: row.recommendations,
            raw_response: row.raw_response,
            model: row.model,
            created_at: parse_datetime(row.created_at.as_str()),
            updated_at: parse_datetime(row.updated_at.as_str()),
        })
    }

    async fn save_ai_insights(
        &self,
        job_id: &str,
        summary: Option<&str>,
        recommendations: Option<&str>,
        raw_response: Option<&str>,
        model: Option<&str>,
    ) -> RepositoryResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query!(
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
            job_id,
            summary,
            recommendations,
            raw_response,
            model,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::from)?;

        Ok(())
    }
}

use super::parse_datetime;
