//! Link repository for the redesigned schema.
//!
//! Links (page edges) have a direct `job_id` foreign key, enabling fast
//! graph queries without joining through page_analysis → analysis_results.

use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool};

use crate::domain::models::{Link, NewLink};
use super::map_link_type;

pub struct LinkRepository {
    pool: SqlitePool,
}

impl LinkRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert multiple links in a batch.
    pub async fn insert_batch(&self, links: &[NewLink]) -> Result<()> {
        if links.is_empty() {
            return Ok(());
        }

        const CHUNK_SIZE: usize = 100;
        let mut tx = self.pool.begin().await?;

        for chunk in links.chunks(CHUNK_SIZE) {
            let mut qb = sqlx::QueryBuilder::new(
                r#"
                INSERT OR IGNORE INTO links (
                    job_id, source_page_id, target_page_id, target_url,
                    link_text, link_type, is_followed, status_code
                ) "#,
            );

            qb.push_values(chunk, |mut b, link| {
                b.push_bind(&link.job_id)
                    .push_bind(&link.source_page_id)
                    .push_bind(&link.target_page_id)
                    .push_bind(&link.target_url)
                    .push_bind(&link.link_text)
                    .push_bind(link.link_type.as_str())
                    .push_bind(link.is_followed)
                    .push_bind(link.status_code);
            });

            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        log::debug!("Inserted {} links", links.len());
        Ok(())
    }

    /// Get all links for a job (FAST: direct FK lookup).
    pub async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<Link>> {
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
        .context("Failed to fetch links for job")?;

        Ok(rows.into_iter().map(|row| row_to_link(&row)).collect())
    }

    /// Get outgoing links from a page.
    pub async fn get_outgoing(&self, source_page_id: &str) -> Result<Vec<Link>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, job_id, source_page_id, target_page_id, target_url,
                link_text, link_type, is_followed, status_code
            FROM links
            WHERE source_page_id = ?
            "#,
        )
        .bind(source_page_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch outgoing links")?;

        Ok(rows.into_iter().map(|row| row_to_link(&row)).collect())
    }

    /// Get incoming links to a page.
    pub async fn get_incoming(&self, target_page_id: &str) -> Result<Vec<Link>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, job_id, source_page_id, target_page_id, target_url,
                link_text, link_type, is_followed, status_code
            FROM links
            WHERE target_page_id = ?
            "#,
        )
        .bind(target_page_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch incoming links")?;

        Ok(rows.into_iter().map(|row| row_to_link(&row)).collect())
    }

    /// Get broken links for a job (status_code >= 400 or NULL).
    pub async fn get_broken(&self, job_id: &str) -> Result<Vec<Link>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, source_page_id, target_page_id, target_url,
                link_text, link_type, is_followed, status_code
            FROM links
            WHERE job_id = ? AND (status_code >= 400 OR status_code IS NULL)
            "#,job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch broken links")?;

        Ok(rows.into_iter().map(|row| row_to_link(&row)).collect())
    }

    /// Get link counts by type for a job.
    pub async fn count_by_type(&self, job_id: &str) -> Result<LinkCounts> {
        let row = sqlx::query(
            r#"
            SELECT 
                SUM(CASE WHEN link_type = 'internal' THEN 1 ELSE 0 END) as internal,
                SUM(CASE WHEN link_type = 'external' THEN 1 ELSE 0 END) as external,
                SUM(CASE WHEN link_type = 'resource' THEN 1 ELSE 0 END) as resource
            FROM links
            WHERE job_id = ?
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count links")?;

        Ok(LinkCounts {
            internal: row.get::<Option<i64>, _>("internal").unwrap_or(0),
            external: row.get::<Option<i64>, _>("external").unwrap_or(0),
            resource: row.get::<Option<i64>, _>("resource").unwrap_or(0),
        })
    }

    /// Get external domains linked from a job.
    pub async fn get_external_domains(&self, job_id: &str) -> Result<Vec<ExternalDomain>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                SUBSTR(target_url, 1, INSTR(SUBSTR(target_url, 9), '/') + 7) as domain,
                COUNT(*) as count
            FROM links
            WHERE job_id = ? AND link_type = 'external'
            GROUP BY domain
            ORDER BY count DESC
            LIMIT 50
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get external domains")?;

        Ok(rows
            .into_iter()
            .map(|row| ExternalDomain {
                domain: row.get("domain"),
                link_count: row.get::<i64, _>("count"),
            })
            .collect())
    }

    /// Update link status codes (after checking links).
    pub async fn update_status_codes(&self, updates: &[(i64, i64)]) -> Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for (link_id, status_code) in updates {
            sqlx::query("UPDATE links SET status_code = ? WHERE id = ?")
                .bind(status_code)
                .bind(link_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

fn row_to_link(row: &sqlx::sqlite::SqliteRow) -> Link {
    Link {
        id: row.get("id"),
        job_id: row.get("job_id"),
        source_page_id: row.get("source_page_id"),
        target_page_id: row.get("target_page_id"),
        target_url: row.get("target_url"),
        link_text: row.get("link_text"),
        link_type: map_link_type(row.get("link_type")),
        is_followed: row.get::<i64, _>("is_followed") != 0,
        status_code: row.get("status_code"),
    }
}

/// Link counts by type.
#[derive(Debug, Clone, Default)]
pub struct LinkCounts {
    pub internal: i64,
    pub external: i64,
    pub resource: i64,
}

impl LinkCounts {
    pub fn total(&self) -> i64 {
        self.internal + self.external + self.resource
    }
}

/// External domain summary.
#[derive(Debug, Clone)]
pub struct ExternalDomain {
    pub domain: String,
    pub link_count: i64,
}
