use crate::contexts::{NewPageQueueItem, PageQueueItem, PageQueueStatus};
use crate::repository::{
    PageQueueRepository as PageQueueRepositoryTrait, RepositoryResult,
};
use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use sqlx::Row;

/// Column lists shared across INSERT and SELECT queries. Defined once
/// to prevent drift when the schema changes.
const INSERT_SQL: &str = r#"
    INSERT INTO page_queue (id, job_id, url, depth, status, created_at, updated_at,
                            cached_html, http_status, cached_load_time_ms, final_url)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#;

const SELECT_COLUMNS: &str =
    "id, job_id, url, depth, status, retry_count, error_message, created_at, updated_at, cached_html, http_status, cached_load_time_ms, final_url";

pub struct PageQueueRepository {
    pool: SqlitePool,
}

impl PageQueueRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Single COUNT helper. Pass `Some(status)` to count rows in that
    /// state, or `None` for the unfiltered total. Replaces three nearly
    /// identical SQL bodies and the previous flag-driven bind dance.
    async fn count_with_status(
        &self,
        job_id: &str,
        status: Option<PageQueueStatus>,
    ) -> RepositoryResult<i64> {
        let row = match status {
            Some(s) => {
                sqlx::query("SELECT COUNT(*) as count FROM page_queue WHERE job_id = ? AND status = ?")
                    .bind(job_id)
                    .bind(s.as_str())
                    .fetch_one(&self.pool)
                    .await?
            }
            None => {
                sqlx::query("SELECT COUNT(*) as count FROM page_queue WHERE job_id = ?")
                    .bind(job_id)
                    .fetch_one(&self.pool)
                    .await?
            }
        };
        Ok(row.get::<i64, _>("count"))
    }
}

#[async_trait]
impl PageQueueRepositoryTrait for PageQueueRepository {
    async fn insert(&self, item: &NewPageQueueItem) -> RepositoryResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let status = "pending";

        sqlx::query(INSERT_SQL)
        .bind(&id)
        .bind(&item.job_id)
        .bind(&item.url)
        .bind(item.depth.as_i64())
        .bind(status)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(&item.cached_html)
        .bind(item.http_status.map(|s| s as i64))
        .bind(item.cached_load_time_ms)
        .bind(&item.final_url)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    async fn insert_batch(&self, items: &[NewPageQueueItem]) -> RepositoryResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for item in items {
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();
            let status = "pending";

            sqlx::query(INSERT_SQL)
            .bind(&id)
            .bind(&item.job_id)
            .bind(&item.url)
            .bind(item.depth.as_i64())
            .bind(status)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(&item.cached_html)
            .bind(item.http_status.map(|s| s as i64))
            .bind(item.cached_load_time_ms)
            .bind(&item.final_url)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn claim_next_pending(
        &self,
        job_id: &str,
    ) -> RepositoryResult<Option<PageQueueItem>> {
        let now = Utc::now();

        // Atomic update: find a pending page and mark it as processing
        let sql = format!(
            "UPDATE page_queue SET status = ?, updated_at = ? \
             WHERE id = (SELECT id FROM page_queue WHERE job_id = ? AND status = 'pending' LIMIT 1) \
             RETURNING {SELECT_COLUMNS}"
        );
        let result = sqlx::query(&sql)
        .bind(PageQueueStatus::Processing.as_str())
        .bind(now.to_rfc3339())
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| map_row_to_item(&row)))
    }

    async fn claim_any_pending(&self) -> RepositoryResult<Option<PageQueueItem>> {
        let now = Utc::now();

        let sql = format!(
            "UPDATE page_queue SET status = ?, updated_at = ? \
             WHERE id = (SELECT id FROM page_queue WHERE status = 'pending' LIMIT 1) \
             RETURNING {SELECT_COLUMNS}"
        );
        let result = sqlx::query(&sql)
        .bind(PageQueueStatus::Processing.as_str())
        .bind(now.to_rfc3339())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| map_row_to_item(&row)))
    }

    async fn update_status(
        &self,
        id: &str,
        status: PageQueueStatus,
    ) -> RepositoryResult<()> {
        let now = Utc::now();
        let status_str = status.as_str();

        // Clear cached_html when completing to free disk space —
        // the analysis has already extracted everything it needs.
        let clear_cache = status == PageQueueStatus::Completed;

        sqlx::query(
            r#"
            UPDATE page_queue
            SET status = ?, updated_at = ?, cached_html = CASE WHEN ? THEN NULL ELSE cached_html END
            WHERE id = ?
            "#,
        )
        .bind(status_str)
        .bind(now.to_rfc3339())
        .bind(clear_cache)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_failed(&self, id: &str, error: &str) -> RepositoryResult<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE page_queue
            SET status = ?, error_message = ?, retry_count = retry_count + 1, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(PageQueueStatus::Failed.as_str())
        .bind(error)
        .bind(now.to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<PageQueueItem>> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM page_queue WHERE job_id = ? ORDER BY created_at ASC"
        );
        let rows = sqlx::query(&sql)
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(map_row_to_item).collect())
    }

    async fn get_by_job_and_status(
        &self,
        job_id: &str,
        status: PageQueueStatus,
    ) -> RepositoryResult<Vec<PageQueueItem>> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM page_queue WHERE job_id = ? AND status = ? ORDER BY created_at ASC"
        );
        let rows = sqlx::query(&sql)
        .bind(job_id)
        .bind(status.as_str())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(map_row_to_item).collect())
    }

    async fn count_pending(&self, job_id: &str) -> RepositoryResult<i64> {
        self.count_with_status(job_id, Some(PageQueueStatus::Pending)).await
    }

    async fn count_completed(&self, job_id: &str) -> RepositoryResult<i64> {
        self.count_with_status(job_id, Some(PageQueueStatus::Completed)).await
    }

    async fn count_total(&self, job_id: &str) -> RepositoryResult<i64> {
        self.count_with_status(job_id, None).await
    }

    async fn delete_by_job_id(&self, job_id: &str) -> RepositoryResult<()> {
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

    async fn reset_processing_to_pending(&self, job_id: &str) -> RepositoryResult<i64> {
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

    async fn is_job_complete(&self, job_id: &str) -> RepositoryResult<bool> {
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
    let status: PageQueueStatus = status_str.parse().unwrap_or_else(|e| {
        tracing::warn!(
            "page_queue: invalid status '{status_str}' ({e}); defaulting to Pending"
        );
        PageQueueStatus::Pending
    });

    let created_at_str: String = row.get("created_at");
    let updated_at_str: String = row.get("updated_at");

    // Decode depth + retry_count as i64 from SQL, then validate via
    // smart constructors. Out-of-range values fall back to safe
    // defaults with a warning (same drift-visibility pattern as the
    // other decoders). `decode_depth` is shared with the page
    // repositories so the fallback rule stays consistent.
    let depth = super::decode_depth(row.get::<i64, _>("depth"));
    let retry_raw: i64 = row.get("retry_count");
    let retry_count = crate::contexts::analysis::RetryCount::new(retry_raw)
        .unwrap_or_else(|e| {
            tracing::warn!(
                "page_queue: invalid retry_count {retry_raw} ({e}); defaulting to 0"
            );
            crate::contexts::analysis::RetryCount::zero()
        });

    PageQueueItem {
        id: row.get("id"),
        job_id: row.get("job_id"),
        url: row.get("url"),
        depth,
        status,
        retry_count,
        error_message: row.get("error_message"),
        created_at: super::parse_datetime(&created_at_str),
        updated_at: super::parse_datetime(&updated_at_str),
        cached_html: row.get("cached_html"),
        http_status: row.try_get::<Option<i64>, _>("http_status")
            .ok()
            .flatten()
            .map(|s| s as u16),
        cached_load_time_ms: row.try_get("cached_load_time_ms").ok().flatten(),
        final_url: row.get("final_url"),
    }
}
