use anyhow::{Context, Result};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{domain::models::PageAnalysisData, service::job_processor::PageEdge};

pub struct PageRepository {
    pool: SqlitePool,
}

impl PageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, page: &PageAnalysisData) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO page_analysis (id, analysis_id, url, title, meta_description, meta_keywords, \
             canonical_url, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, \
             internal_links, external_links, load_time, status_code, content_size, mobile_friendly, \
             has_structured_data, lighthouse_performance, lighthouse_accessibility, \
             lighthouse_best_practices, lighthouse_seo, created_at, headings, images, links) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&page.analysis_id)
        .bind(&page.url)
        .bind(&page.title)
        .bind(&page.meta_description)
        .bind(&page.meta_keywords)
        .bind(&page.canonical_url)
        .bind(page.h1_count)
        .bind(page.h2_count)
        .bind(page.h3_count)
        .bind(page.word_count)
        .bind(page.image_count)
        .bind(page.images_without_alt)
        .bind(page.internal_links)
        .bind(page.external_links)
        .bind(page.load_time)
        .bind(page.status_code)
        .bind(page.content_size)
        .bind(page.mobile_friendly)
        .bind(page.has_structured_data)
        .bind(page.lighthouse_performance)
        .bind(page.lighthouse_accessibility)
        .bind(page.lighthouse_best_practices)
        .bind(page.lighthouse_seo)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(serde_json::to_string(&page.headings).unwrap_or_default())
        .bind(serde_json::to_string(&page.images).unwrap_or_default())
        .bind(serde_json::to_string(&page.detailed_links).unwrap_or_default())
        .execute(&self.pool)
        .await
        .context("Failed to insert page analysis")?;

        Ok(id)
    }

    pub async fn insert_batch(&self, pages: &[PageAnalysisData]) -> Result<()> {
        if pages.is_empty() {
            return Ok(());
        }

        const CHUNK_SIZE: usize = 50;
        let mut tx = self.pool.begin().await?;

        for chunk in pages.chunks(CHUNK_SIZE) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO page_analysis (id, analysis_id, url, title, meta_description, meta_keywords, \
                 canonical_url, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, \
                 internal_links, external_links, load_time, status_code, content_size, mobile_friendly, \
                 has_structured_data, lighthouse_performance, lighthouse_accessibility, \
                 lighthouse_best_practices, lighthouse_seo, created_at, headings, images, links) "
            );

            query_builder.push_values(chunk, |mut b, page| {
                b.push_bind(Uuid::new_v4().to_string()) // Generate new ID for each page
                    .push_bind(&page.analysis_id)
                    .push_bind(&page.url)
                    .push_bind(&page.title)
                    .push_bind(&page.meta_description)
                    .push_bind(&page.meta_keywords)
                    .push_bind(&page.canonical_url)
                    .push_bind(page.h1_count)
                    .push_bind(page.h2_count)
                    .push_bind(page.h3_count)
                    .push_bind(page.word_count)
                    .push_bind(page.image_count)
                    .push_bind(page.images_without_alt)
                    .push_bind(page.internal_links)
                    .push_bind(page.external_links)
                    .push_bind(page.load_time)
                    .push_bind(page.status_code)
                    .push_bind(page.content_size)
                    .push_bind(page.mobile_friendly)
                    .push_bind(page.has_structured_data)
                    .push_bind(page.lighthouse_performance)
                    .push_bind(page.lighthouse_accessibility)
                    .push_bind(page.lighthouse_best_practices)
                    .push_bind(page.lighthouse_seo)
                    .push_bind(chrono::Utc::now().to_rfc3339())
                    .push_bind(serde_json::to_string(&page.headings).unwrap_or_default())
                    .push_bind(serde_json::to_string(&page.images).unwrap_or_default())
                    .push_bind(serde_json::to_string(&page.detailed_links).unwrap_or_default());
            });
            query_builder
                .build()
                .execute(&mut *tx)
                .await
                .context("Failed to insert page analysis chunk")?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn insert_edges_batch(&self, edges: &[PageEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }
        const CHUNK_SIZE: usize = 50;
        let mut tx = self.pool.begin().await?;

        for chunk in edges.chunks(CHUNK_SIZE) {
            let mut qb = sqlx::QueryBuilder::new(
                "INSERT INTO page_edge (from_page_id, to_url, status_code) ",
            );
            qb.push_values(chunk, |mut b, edge| {
                b.push_bind(&edge.from_page_id)
                    .push_bind(&edge.to_url)
                    .push_bind(edge.status_code as i32);
            });
            qb.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
