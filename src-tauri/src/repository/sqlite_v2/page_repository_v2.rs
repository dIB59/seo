//! Page repository for the redesigned schema.
//!
//! Pages have a direct `job_id` foreign key, eliminating the need to
//! join through analysis_results.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::domain::models_v2::{LighthouseData, Page, PageInfo};

pub struct PageRepositoryV2 {
    pool: SqlitePool,
}

impl PageRepositoryV2 {
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

        sqlx::query(
            r#"
            INSERT INTO pages (
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes, crawled_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&page.job_id)
        .bind(&page.url)
        .bind(page.depth)
        .bind(page.status_code)
        .bind(&page.content_type)
        .bind(&page.title)
        .bind(&page.meta_description)
        .bind(&page.canonical_url)
        .bind(&page.robots_meta)
        .bind(page.word_count)
        .bind(page.load_time_ms)
        .bind(page.response_size_bytes)
        .bind(page.crawled_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .context("Failed to insert page")?;

        Ok(id)
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

            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        log::debug!("Inserted {} pages", pages.len());
        Ok(())
    }

    /// Get all pages for a job (FAST: direct FK lookup).
    pub async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<Page>> {
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
        .context("Failed to fetch pages for job")?;

        Ok(rows.into_iter().map(|row| row_to_page(&row)).collect())
    }

    /// Get page info with issue counts for listing.
    pub async fn get_info_by_job_id(&self, job_id: &str) -> Result<Vec<PageInfo>> {
        let rows = sqlx::query(
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
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch page info for job")?;

        Ok(rows
            .into_iter()
            .map(|row| PageInfo {
                id: row.get("id"),
                url: row.get("url"),
                title: row.get("title"),
                status_code: row.get("status_code"),
                load_time_ms: row.get("load_time_ms"),
                issue_count: row.get::<i64, _>("issue_count"),
            })
            .collect())
    }

    /// Get a single page by ID.
    pub async fn get_by_id(&self, page_id: &str) -> Result<Page> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes, crawled_at
            FROM pages
            WHERE id = ?
            "#,
        )
        .bind(page_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch page")?;

        Ok(row_to_page(&row))
    }

    /// Get page count for a job (FAST: uses index).
    pub async fn count_by_job_id(&self, job_id: &str) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM pages WHERE job_id = ?")
            .bind(job_id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to count pages")?;

        Ok(row.get::<i64, _>("count"))
    }

    /// Insert Lighthouse data for a page.
    pub async fn insert_lighthouse(&self, data: &LighthouseData) -> Result<()> {
        sqlx::query(
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
        )
        .bind(&data.page_id)
        .bind(data.performance_score)
        .bind(data.accessibility_score)
        .bind(data.best_practices_score)
        .bind(data.seo_score)
        .bind(data.first_contentful_paint_ms)
        .bind(data.largest_contentful_paint_ms)
        .bind(data.total_blocking_time_ms)
        .bind(data.cumulative_layout_shift)
        .bind(data.speed_index)
        .bind(data.time_to_interactive_ms)
        .bind(&data.raw_json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .context("Failed to insert lighthouse data")?;

        Ok(())
    }

    /// Get Lighthouse data for pages in a job.
    pub async fn get_lighthouse_by_job_id(&self, job_id: &str) -> Result<Vec<LighthouseData>> {
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
}

fn row_to_page(row: &sqlx::sqlite::SqliteRow) -> Page {
    Page {
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
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
