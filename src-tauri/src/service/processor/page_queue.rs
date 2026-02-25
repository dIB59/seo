use crate::domain::{NewPageQueueItem, PageQueueItem, PageQueueStatus};
use crate::repository::PageQueueRepository;
use anyhow::Result;
use std::sync::Arc;

/// Manages the page queue for a job.
/// Provides high-level operations for inserting, claiming, and updating pages.
pub struct PageQueueManager {
    repo: Arc<dyn PageQueueRepository>,
}

impl PageQueueManager {
    /// Create a new page queue manager.
    pub fn new(repo: Arc<dyn PageQueueRepository>) -> Self {
        Self { repo }
    }

    /// Insert discovered URLs into the page queue.
    /// This is called after the discovery phase.
    pub async fn insert_discovered_urls(
        &self,
        job_id: &str,
        urls: &[String],
        max_depth: i64,
    ) -> Result<usize> {
        let items: Vec<NewPageQueueItem> = urls
            .iter()
            .map(|url| NewPageQueueItem::new(job_id, url, max_depth))
            .collect();

        let count = items.len();
        self.repo.insert_batch(&items).await?;
        
        tracing::info!(
            "Inserted {} pages into queue for job {}",
            count,
            job_id
        );
        
        Ok(count)
    }

    /// Claim the next pending page for a job.
    /// Returns None if no pending pages are available.
    pub async fn claim_next_page(&self, job_id: &str) -> Result<Option<PageQueueItem>> {
        self.repo.claim_next_pending(job_id).await
    }

    /// Mark a page as completed.
    pub async fn mark_completed(&self, id: &str) -> Result<()> {
        self.repo.update_status(id, PageQueueStatus::Completed).await
    }

    /// Mark a page as failed with an error message.
    pub async fn mark_failed(&self, id: &str, error: &str) -> Result<()> {
        self.repo.mark_failed(id, error).await
    }

    /// Get the count of pending pages for a job.
    pub async fn pending_count(&self, job_id: &str) -> Result<i64> {
        self.repo.count_pending(job_id).await
    }

    /// Get the count of completed pages for a job.
    pub async fn completed_count(&self, job_id: &str) -> Result<i64> {
        self.repo.count_completed(job_id).await
    }

    /// Get the total count of pages for a job.
    pub async fn total_count(&self, job_id: &str) -> Result<i64> {
        self.repo.count_total(job_id).await
    }

    /// Check if all pages for a job are complete.
    pub async fn is_complete(&self, job_id: &str) -> Result<bool> {
        self.repo.is_job_complete(job_id).await
    }

    /// Reset any processing pages back to pending (for recovery after crash).
    pub async fn reset_processing_pages(&self, job_id: &str) -> Result<i64> {
        self.repo.reset_processing_to_pending(job_id).await
    }

    /// Get progress information for a job.
    pub async fn get_progress(&self, job_id: &str) -> Result<PageQueueProgress> {
        let pending = self.repo.count_pending(job_id).await?;
        let completed = self.repo.count_completed(job_id).await?;
        let total = self.repo.count_total(job_id).await?;
        
        Ok(PageQueueProgress {
            pending,
            completed,
            total,
            percentage: if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        })
    }
}

/// Progress information for a job's page queue.
#[derive(Debug, Clone, Copy)]
pub struct PageQueueProgress {
    pub pending: i64,
    pub completed: i64,
    pub total: i64,
    pub percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock repository for testing
    struct MockPageQueueRepo {
        pending_count: i64,
        completed_count: i64,
        total_count: i64,
    }

    #[async_trait]
    impl PageQueueRepository for MockPageQueueRepo {
        async fn insert(&self, _item: &NewPageQueueItem) -> Result<String> {
            Ok(uuid::Uuid::new_v4().to_string())
        }

        async fn insert_batch(&self, _items: &[NewPageQueueItem]) -> Result<()> {
            Ok(())
        }

        async fn claim_next_pending(&self, _job_id: &str) -> Result<Option<PageQueueItem>> {
            Ok(None)
        }

        async fn claim_any_pending(&self) -> Result<Option<PageQueueItem>> {
            Ok(None)
        }

        async fn update_status(&self, _id: &str, _status: PageQueueStatus) -> Result<()> {
            Ok(())
        }

        async fn mark_failed(&self, _id: &str, _error: &str) -> Result<()> {
            Ok(())
        }

        async fn get_by_job_id(&self, _job_id: &str) -> Result<Vec<PageQueueItem>> {
            Ok(vec![])
        }

        async fn get_by_job_and_status(
            &self,
            _job_id: &str,
            _status: PageQueueStatus,
        ) -> Result<Vec<PageQueueItem>> {
            Ok(vec![])
        }

        async fn count_pending(&self, _job_id: &str) -> Result<i64> {
            Ok(self.pending_count)
        }

        async fn count_completed(&self, _job_id: &str) -> Result<i64> {
            Ok(self.completed_count)
        }

        async fn count_total(&self, _job_id: &str) -> Result<i64> {
            Ok(self.total_count)
        }

        async fn delete_by_job_id(&self, _job_id: &str) -> Result<()> {
            Ok(())
        }

        async fn reset_processing_to_pending(&self, _job_id: &str) -> Result<i64> {
            Ok(0)
        }

        async fn is_job_complete(&self, _job_id: &str) -> Result<bool> {
            Ok(self.pending_count == 0)
        }
    }

    #[tokio::test]
    async fn test_get_progress() {
        let repo = Arc::new(MockPageQueueRepo {
            pending_count: 5,
            completed_count: 15,
            total_count: 20,
        });
        
        let manager = PageQueueManager::new(repo);
        let progress = manager.get_progress("test-job").await.unwrap();
        
        assert_eq!(progress.pending, 5);
        assert_eq!(progress.completed, 15);
        assert_eq!(progress.total, 20);
        assert_eq!(progress.percentage, 75.0);
    }

    #[tokio::test]
    async fn test_is_complete() {
        let repo = Arc::new(MockPageQueueRepo {
            pending_count: 0,
            completed_count: 10,
            total_count: 10,
        });
        
        let manager = PageQueueManager::new(repo);
        assert!(manager.is_complete("test-job").await.unwrap());
    }

    #[tokio::test]
    async fn test_not_complete() {
        let repo = Arc::new(MockPageQueueRepo {
            pending_count: 5,
            completed_count: 5,
            total_count: 10,
        });
        
        let manager = PageQueueManager::new(repo);
        assert!(!manager.is_complete("test-job").await.unwrap());
    }
}
