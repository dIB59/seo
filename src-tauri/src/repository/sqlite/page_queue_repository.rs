use crate::contexts::{NewPageQueueItem, PageQueueItem, PageQueueStatus};
use crate::repository::PageQueueRepository as PageQueueRepositoryTrait;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use sqlx::Row;

pub struct PageQueueRepository {
    pool: SqlitePool,
}

impl PageQueueRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn count_with_status(&self, job_id: &str, status: Option<&str>) -> Result<i64> {
        let (sql, bind_status) = match status {
            Some(_) => (
                "SELECT COUNT(*) as count FROM page_queue WHERE job_id = ? AND status = ?",
                true,
            ),
            None => (
                "SELECT COUNT(*) as count FROM page_queue WHERE job_id = ?",
                false,
            ),
        };
        let mut q = sqlx::query(sql).bind(job_id);
        if bind_status {
            q = q.bind(status.unwrap());
        }
        let row = q.fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>("count"))
    }
}

#[async_trait]
impl PageQueueRepositoryTrait for PageQueueRepository {
    async fn insert(&self, item: &NewPageQueueItem) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let status = "pending";

        sqlx::query(
            r#"
            INSERT INTO page_queue (id, job_id, url, depth, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&item.job_id)
        .bind(&item.url)
        .bind(item.depth)
        .bind(status)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    async fn insert_batch(&self, items: &[NewPageQueueItem]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for item in items {
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();
            let status = "pending";

            sqlx::query(
                r#"
                INSERT INTO page_queue (id, job_id, url, depth, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&id)
            .bind(&item.job_id)
            .bind(&item.url)
            .bind(item.depth)
            .bind(status)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn claim_next_pending(&self, job_id: &str) -> Result<Option<PageQueueItem>> {
        let now = Utc::now();
        let processing = "processing";

        // Atomic update: find a pending page and mark it as processing
        let result = sqlx::query(
            r#"
            UPDATE page_queue
            SET status = ?, updated_at = ?
            WHERE id = (
                SELECT id FROM page_queue
                WHERE job_id = ? AND status = 'pending'
                LIMIT 1
            )
            RETURNING id, job_id, url, depth, status, retry_count, error_message, created_at, updated_at
            "#,
        )
        .bind(processing)
        .bind(now.to_rfc3339())
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| map_row_to_item(&row)))
    }

    async fn claim_any_pending(&self) -> Result<Option<PageQueueItem>> {
        let now = Utc::now();
        let processing = "processing";

        // Atomic update: find any pending page and mark it as processing
        let result = sqlx::query(
            r#"
            UPDATE page_queue
            SET status = ?, updated_at = ?
            WHERE id = (
                SELECT id FROM page_queue
                WHERE status = 'pending'
                LIMIT 1
            )
            RETURNING id, job_id, url, depth, status, retry_count, error_message, created_at, updated_at
            "#,
        )
        .bind(processing)
        .bind(now.to_rfc3339())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| map_row_to_item(&row)))
    }

    async fn update_status(&self, id: &str, status: PageQueueStatus) -> Result<()> {
        let now = Utc::now();
        let status_str = status.as_str();

        sqlx::query(
            r#"
            UPDATE page_queue
            SET status = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status_str)
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_failed(&self, id: &str, error: &str) -> Result<()> {
        let now = Utc::now();
        let failed = "failed";

        sqlx::query(
            r#"
            UPDATE page_queue
            SET status = ?, error_message = ?, retry_count = retry_count + 1, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(failed)
        .bind(error)
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<PageQueueItem>> {
        let rows = sqlx::query(
            r#"
            SELECT id, job_id, url, depth, status, retry_count, error_message, created_at, updated_at
            FROM page_queue
            WHERE job_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(map_row_to_item).collect())
    }

    async fn get_by_job_and_status(
        &self,
        job_id: &str,
        status: PageQueueStatus,
    ) -> Result<Vec<PageQueueItem>> {
        let status_str = status.as_str();

        let rows = sqlx::query(
            r#"
            SELECT id, job_id, url, depth, status, retry_count, error_message, created_at, updated_at
            FROM page_queue
            WHERE job_id = ? AND status = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .bind(status_str)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(map_row_to_item).collect())
    }

    async fn count_pending(&self, job_id: &str) -> Result<i64> {
        self.count_with_status(job_id, Some("pending")).await
    }

    async fn count_completed(&self, job_id: &str) -> Result<i64> {
        self.count_with_status(job_id, Some("completed")).await
    }

    async fn count_total(&self, job_id: &str) -> Result<i64> {
        self.count_with_status(job_id, None).await
    }

    async fn delete_by_job_id(&self, job_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM page_queue
            WHERE job_id = ?
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn reset_processing_to_pending(&self, job_id: &str) -> Result<i64> {
        let now = Utc::now();
        let pending = "pending";

        let result = sqlx::query(
            r#"
            UPDATE page_queue
            SET status = ?, updated_at = ?
            WHERE job_id = ? AND status = 'processing'
            "#,
        )
        .bind(pending)
        .bind(now.to_rfc3339())
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn is_job_complete(&self, job_id: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM page_queue
            WHERE job_id = ? AND status IN ('pending', 'processing')
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get::<i64, _>("count") == 0)
    }
}

fn map_row_to_item(row: &sqlx::sqlite::SqliteRow) -> PageQueueItem {
    let status_str: String = row.get("status");
    let status: PageQueueStatus = status_str.parse().unwrap_or(PageQueueStatus::Pending);

    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");

    PageQueueItem {
        id: row.get("id"),
        job_id: row.get("job_id"),
        url: row.get("url"),
        depth: row.get("depth"),
        status,
        retry_count: row.get("retry_count"),
        error_message: row.get("error_message"),
        created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at_str.parse().unwrap_or_else(|_| Utc::now()),
    }
}
