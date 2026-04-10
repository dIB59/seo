use chrono::Utc;
use sqlx::SqlitePool;

use crate::contexts::analysis::{LighthouseData, NewHeading, NewImage, Page, PageInfo};
use crate::repository::{PageRepository as PageRepositoryTrait, RepositoryError, RepositoryResult};
use async_trait::async_trait;
use super::decode_extracted_data;

/// Use the page's existing id, or mint a fresh UUID if it's empty.
/// Both `insert` and `insert_batch` need this logic.
fn page_id_or_new(page: &Page) -> String {
    if page.id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        page.id.clone()
    }
}

/// Serialize a page's `extracted_data` map to JSON for storage. Falls
/// back to `"{}"` on serialization failure rather than failing the
/// whole insert — the map is `HashMap<String, serde_json::Value>` so
/// this practically can't fail, but the safety net is preserved from
/// the original code and now lives in one place.
fn encode_extracted_data(page: &Page) -> String {
    serde_json::to_string(&page.extracted_data).unwrap_or_else(|_| "{}".to_string())
}

pub struct PageRepository {
    pool: SqlitePool,
}

impl PageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PageRepositoryTrait for PageRepository {
    async fn insert(&self, page: &Page) -> RepositoryResult<String> {
        let id = page_id_or_new(page);
        let crawled_at_str = page.crawled_at.to_rfc3339();
        let extracted_data_json = encode_extracted_data(page);
        let depth_raw = page.depth.as_i64();

        let row = sqlx::query!(
            r#"
            INSERT INTO pages (
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes,
                has_viewport, has_structured_data, crawled_at, extracted_data
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                has_viewport = excluded.has_viewport,
                has_structured_data = excluded.has_structured_data,
                crawled_at = excluded.crawled_at,
                extracted_data = excluded.extracted_data
            RETURNING id
            "#,
            id,
            page.job_id,
            page.url,
            depth_raw,
            page.status_code,
            page.content_type,
            page.title,
            page.meta_description,
            page.canonical_url,
            page.robots_meta,
            page.word_count,
            page.load_time_ms,
            page.response_size_bytes,
            page.has_viewport,
            page.has_structured_data,
            crawled_at_str,
            extracted_data_json
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }

    async fn insert_batch(&self, pages: &[Page]) -> RepositoryResult<()> {
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
                    word_count, load_time_ms, response_size_bytes,
                    has_viewport, has_structured_data, crawled_at, extracted_data
                ) "#,
            );

            qb.push_values(chunk, |mut b, page| {
                let id = page_id_or_new(page);
                let extracted_data_json = encode_extracted_data(page);

                b.push_bind(id)
                    .push_bind(&page.job_id)
                    .push_bind(&page.url)
                    .push_bind(page.depth.as_i64())
                    .push_bind(page.status_code)
                    .push_bind(&page.content_type)
                    .push_bind(&page.title)
                    .push_bind(&page.meta_description)
                    .push_bind(&page.canonical_url)
                    .push_bind(&page.robots_meta)
                    .push_bind(page.word_count)
                    .push_bind(page.load_time_ms)
                    .push_bind(page.response_size_bytes)
                    .push_bind(page.has_viewport)
                    .push_bind(page.has_structured_data)
                    .push_bind(page.crawled_at.to_rfc3339())
                    .push_bind(extracted_data_json);
            });

            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        tracing::debug!("Inserted {} pages", pages.len());
        Ok(())
    }

    async fn get_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<Page>> {
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
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let extracted_data = decode_extracted_data(&row.extracted_data);
                super::page_from_row!(row, extracted_data)
            })
            .collect())
    }

    async fn get_info_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<PageInfo>> {
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
        .await?;

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

    async fn get_by_id(&self, page_id: &str) -> RepositoryResult<Page> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes,
                has_viewport, has_structured_data, crawled_at, extracted_data
            FROM pages
            WHERE id = ?
            "#,
            page_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::not_found("page", page_id),
            other => RepositoryError::from(other),
        })?;

        let extracted_data = decode_extracted_data(&row.extracted_data);
        Ok(super::page_from_row!(row, extracted_data))
    }

    async fn replace_headings(
        &self,
        page_id: &str,
        headings: &[NewHeading],
    ) -> RepositoryResult<()> {
        sqlx::query!("DELETE FROM page_headings WHERE page_id = ?", page_id)
            .execute(&self.pool)
            .await?;

        if headings.is_empty() {
            return Ok(());
        }

        let mut qb =
            sqlx::QueryBuilder::new("INSERT INTO page_headings (page_id, level, text, position) ");

        qb.push_values(headings, |mut b, h| {
            b.push_bind(&h.page_id)
                .push_bind(h.level)
                .push_bind(&h.text)
                .push_bind(h.position);
        });

        qb.build().execute(&self.pool).await?;

        Ok(())
    }

    async fn replace_images(
        &self,
        page_id: &str,
        images: &[NewImage],
    ) -> RepositoryResult<()> {
        sqlx::query!("DELETE FROM page_images WHERE page_id = ?", page_id)
            .execute(&self.pool)
            .await?;

        if images.is_empty() {
            return Ok(());
        }

        let mut qb = sqlx::QueryBuilder::new(
            "INSERT INTO page_images (page_id, src, alt, width, height, loading, is_decorative) ",
        );

        qb.push_values(images, |mut b, i| {
            b.push_bind(&i.page_id)
                .push_bind(&i.src)
                .push_bind(&i.alt)
                .push_bind(i.width)
                .push_bind(i.height)
                .push_bind(&i.loading)
                .push_bind(if i.is_decorative { 1 } else { 0 });
        });

        qb.build().execute(&self.pool).await?;

        Ok(())
    }

    async fn count_by_job_id(&self, job_id: &str) -> RepositoryResult<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM pages WHERE job_id = ?",
            job_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count as i64)
    }

    async fn insert_lighthouse(&self, data: &LighthouseData) -> RepositoryResult<()> {
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
        .await?;

        Ok(())
    }

    async fn get_lighthouse_by_job_id(
        &self,
        job_id: &str,
    ) -> RepositoryResult<Vec<LighthouseData>> {
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
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| super::lighthouse_data_from_row!(row))
            .collect())
    }
}

