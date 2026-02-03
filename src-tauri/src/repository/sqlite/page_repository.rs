//! Page repository for the redesigned schema.
//!
//! Pages have a direct `job_id` foreign key, eliminating the need to
//! join through analysis_results.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::models::{LighthouseData, Page, PageInfo};

pub struct PageRepository {
    pool: SqlitePool,
}

impl PageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert a single page.
    pub async fn insert(&self, page: &Page) -> Result<String> {
        let id = if page.id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            page.id.clone()
        };

        let crawled_at_str = page.crawled_at.to_rfc3339();
        let row = sqlx::query!(
            r#"
            INSERT INTO pages (
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes, crawled_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(job_id, url) DO UPDATE SET
                depth = excluded.depth,
                status_code = excluded.status_code,
                content_type = excluded.content_type,
                title = excluded.title,
                meta_description = excluded.meta_description,
                canonical_url = excluded.canonical_url,
                robots_meta = excluded.robots_meta,
                word_count = excluded.word_count,
                load_time_ms = excluded.load_time_ms,
                response_size_bytes = excluded.response_size_bytes,
                crawled_at = excluded.crawled_at
            RETURNING id
            "#,
            id,
            page.job_id,
            page.url,
            page.depth,
            page.status_code,
            page.content_type,
            page.title,
            page.meta_description,
            page.canonical_url,
            page.robots_meta,
            page.word_count,
            page.load_time_ms,
            page.response_size_bytes,
            crawled_at_str
        )
        .fetch_one(&self.pool)
        .await
        .with_context(|| format!("Failed to upsert page (job_id={}, url={})", page.job_id, page.url))?;

        Ok(row.id)
    }

    /// Insert multiple pages in a batch.
    pub async fn insert_batch(&self, pages: &[Page]) -> Result<()> {
        if pages.is_empty() {
            return Ok(());
        }

        const CHUNK_SIZE: usize = 100;
        let mut tx = self.pool.begin().await?;

        for chunk in pages.chunks(CHUNK_SIZE) {
            let mut qb = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO pages (
                    id, job_id, url, depth, status_code, content_type,
                    title, meta_description, canonical_url, robots_meta,
                    word_count, load_time_ms, response_size_bytes, crawled_at
                ) "#,
            );

            qb.push_values(chunk, |mut b, page| {
                let id = if page.id.is_empty() {
                    uuid::Uuid::new_v4().to_string()
                } else {
                    page.id.clone()
                };
                b.push_bind(id)
                    .push_bind(&page.job_id)
                    .push_bind(&page.url)
                    .push_bind(page.depth)
                    .push_bind(page.status_code)
                    .push_bind(&page.content_type)
                    .push_bind(&page.title)
                    .push_bind(&page.meta_description)
                    .push_bind(&page.canonical_url)
                    .push_bind(&page.robots_meta)
                    .push_bind(page.word_count)
                    .push_bind(page.load_time_ms)
                    .push_bind(page.response_size_bytes)
                    .push_bind(page.crawled_at.to_rfc3339());
            });

            qb.build().execute(&mut *tx).await.context("Failed to batch insert pages")?;
        }

        tx.commit().await?;
        log::debug!("Inserted {} pages", pages.len());
        Ok(())
    }

    /// Get all pages for a job (FAST: direct FK lookup).
    pub async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<Page>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes, crawled_at
            FROM pages
            WHERE job_id = ?
            ORDER BY depth ASC, url ASC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pages for job")?;

        Ok(rows
            .into_iter()
            .map(|row| Page {
                id: row.id,
                job_id: row.job_id,
                url: row.url,
                depth: row.depth,
                status_code: row.status_code,
                content_type: row.content_type,
                title: row.title,
                meta_description: row.meta_description,
                canonical_url: row.canonical_url,
                robots_meta: row.robots_meta,
                word_count: row.word_count,
                load_time_ms: row.load_time_ms,
                response_size_bytes: row.response_size_bytes,
                crawled_at: parse_datetime(row.crawled_at.as_str()),
            })
            .collect())
    }

    /// Get page info with issue counts for listing.
    pub async fn get_info_by_job_id(&self, job_id: &str) -> Result<Vec<PageInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                p.id, p.url, p.title, p.status_code, p.load_time_ms,
                COUNT(i.id) as issue_count
            FROM pages p
            LEFT JOIN issues i ON i.page_id = p.id
            WHERE p.job_id = ?
            GROUP BY p.id
            ORDER BY issue_count DESC, p.url ASC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch page info for job")?;

        Ok(rows
            .into_iter()
            .map(|row| PageInfo {
                id: row.id,
                url: row.url,
                title: row.title,
                status_code: row.status_code,
                load_time_ms: row.load_time_ms,
                issue_count: row.issue_count,
            })
            .collect())
    }

    /// Get a single page by ID.
    pub async fn get_by_id(&self, page_id: &str) -> Result<Page> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes, crawled_at
            FROM pages
            WHERE id = ?
            "#,
            page_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch page")?;

        Ok(Page {
            id: row.id,
            job_id: row.job_id,
            url: row.url,
            depth: row.depth,
            status_code: row.status_code,
            content_type: row.content_type,
            title: row.title,
            meta_description: row.meta_description,
            canonical_url: row.canonical_url,
            robots_meta: row.robots_meta,
            word_count: row.word_count,
            load_time_ms: row.load_time_ms,
            response_size_bytes: row.response_size_bytes,
            crawled_at: parse_datetime(row.crawled_at.as_str()),
        })
    }

    /// Get page count for a job (FAST: uses index).
    pub async fn count_by_job_id(&self, job_id: &str) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM pages WHERE job_id = ?",
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count pages")?;

        Ok(row.count as i64)
    }

    /// Insert Lighthouse data for a page.
    pub async fn insert_lighthouse(&self, data: &LighthouseData) -> Result<()> {
        let created_at = Utc::now().to_rfc3339();
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO page_lighthouse (
                page_id, performance_score, accessibility_score, 
                best_practices_score, seo_score,
                first_contentful_paint_ms, largest_contentful_paint_ms,
                total_blocking_time_ms, cumulative_layout_shift,
                speed_index, time_to_interactive_ms, raw_json, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            data.page_id,
            data.performance_score,
            data.accessibility_score,
            data.best_practices_score,
            data.seo_score,
            data.first_contentful_paint_ms,
            data.largest_contentful_paint_ms,
            data.total_blocking_time_ms,
            data.cumulative_layout_shift,
            data.speed_index,
            data.time_to_interactive_ms,
            data.raw_json,
            created_at
        )
        .execute(&self.pool)
        .await
        .context("Failed to insert lighthouse data")?;

        Ok(())
    }

    /// Get Lighthouse data for pages in a job.
    pub async fn get_lighthouse_by_job_id(&self, job_id: &str) -> Result<Vec<LighthouseData>> {
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
        .context("Failed to fetch lighthouse data")?;

        Ok(rows
            .into_iter()
            .map(|row| LighthouseData {
                page_id: row.page_id,
                performance_score: row.performance_score,
                accessibility_score: row.accessibility_score,
                best_practices_score: row.best_practices_score,
                seo_score: row.seo_score,
                first_contentful_paint_ms: row.first_contentful_paint_ms,
                largest_contentful_paint_ms: row.largest_contentful_paint_ms,
                total_blocking_time_ms: row.total_blocking_time_ms,
                cumulative_layout_shift: row.cumulative_layout_shift,
                speed_index: row.speed_index,
                time_to_interactive_ms: row.time_to_interactive_ms,
                raw_json: row.raw_json,
            })
            .collect())
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
