use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub struct AiRepository {
    pool: SqlitePool,
}

impl AiRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get cached AI insights for a job (V2 schema)
    /// Note: V2 stores structured insights. This returns the summary for backward compatibility.
    pub async fn get_ai_insights(&self, job_id: &str) -> Result<Option<String>> {
        let result =
            sqlx::query_scalar::<_, String>("SELECT summary FROM ai_insights WHERE job_id = ?")
                .bind(job_id)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to get ai insights from database")?;

        Ok(result)
    }

    /// Save AI insights to the database (V2 schema)
    /// For backward compatibility, stores insights as the summary field.
    pub async fn save_ai_insights(&self, job_id: &str, insights: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO ai_insights (job_id, summary, created_at, updated_at) VALUES (?, ?, datetime('now'), datetime('now'))
             ON CONFLICT(job_id) DO UPDATE SET summary = ?, updated_at = datetime('now')",
            job_id,
            insights,
            insights
        )
        .execute(&self.pool)
        .await
        .context("Failed to save ai insights to database")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixtures;

    #[tokio::test]
    async fn test_ai_insights_returns_none_when_not_cached() {
        let pool = fixtures::setup_test_db().await;
        let repo = AiRepository::new(pool);

        let result = repo.get_ai_insights("nonexistent_job").await.unwrap();
        assert!(result.is_none(), "Should return None for non-cached job");
    }

    /// Helper to create a valid jobs record for FK constraint (V2 schema)
    async fn create_test_job(pool: &SqlitePool, id: &str) {
        sqlx::query(
            "INSERT INTO jobs (id, url, status, created_at, updated_at) 
             VALUES (?, 'https://test.com', 'completed', datetime('now'), datetime('now'))",
        )
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_and_get_ai_insights() {
        let pool = fixtures::setup_test_db().await;
        let repo = AiRepository::new(pool.clone());

        // Create the job record first to satisfy FK constraint
        create_test_job(&pool, "job_123").await;

        repo.save_ai_insights("job_123", "These are AI insights")
            .await
            .unwrap();

        let result = repo.get_ai_insights("job_123").await.unwrap();
        assert_eq!(result, Some("These are AI insights".to_string()));
    }

    #[tokio::test]
    async fn test_save_ai_insights_updates_existing() {
        let pool = fixtures::setup_test_db().await;
        let repo = AiRepository::new(pool.clone());

        // Create the job record first to satisfy FK constraint
        create_test_job(&pool, "job_456").await;

        repo.save_ai_insights("job_456", "Original insights")
            .await
            .unwrap();
        repo.save_ai_insights("job_456", "Updated insights")
            .await
            .unwrap();

        let result = repo.get_ai_insights("job_456").await.unwrap();
        assert_eq!(
            result,
            Some("Updated insights".to_string()),
            "Should update existing insights"
        );
    }
}
