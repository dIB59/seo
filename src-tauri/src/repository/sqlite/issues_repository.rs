//! # DEPRECATED - V1 Repository
//!
//! This module contains the V1 SQLite repository implementation.
//! It has been superseded by the V2 schema (migration 0018+).
//!
//! **Warning:** This code uses runtime SQL queries because the V1 table
//! (seo_issues) no longer exists in the schema after migration 0018.
//!
//! This repository will only work at runtime if V1 tables still exist
//! in the database (e.g., for legacy data migration purposes).

use anyhow::Result;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::domain::models::SeoIssue;

#[derive(Clone)]
pub struct IssuesRepository {
    pool: SqlitePool,
}

impl IssuesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_batch(&self, issues: &[SeoIssue]) -> Result<()> {
        if issues.is_empty() {
            return Ok(());
        }

        const CHUNK_SIZE: usize = 50;
        let mut tx = self.pool.begin().await?;

        for chunk in issues.chunks(CHUNK_SIZE) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO seo_issues (id, page_id, type, title, description, page_url, element, line_number, recommendation) "
            );

            query_builder.push_values(chunk, |mut b, issue| {
                b.push_bind(Uuid::new_v4().to_string())
                    .push_bind(&issue.page_id)
                    .push_bind(issue.issue_type.as_str())
                    .push_bind(&issue.title)
                    .push_bind(&issue.description)
                    .push_bind(&issue.page_url)
                    .push_bind(&issue.element)
                    .push_bind(issue.line_number)
                    .push_bind(&issue.recommendation);
            });

            query_builder.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
