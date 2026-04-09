use dashmap::DashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct JobCanceler {
    cancel_map: Arc<DashMap<String, CancellationToken>>,
}

impl JobCanceler {
    pub fn new() -> Self {
        Self {
            cancel_map: Arc::new(DashMap::with_capacity(10)),
        }
    }

    pub fn get_token(&self, job_id: &str) -> CancellationToken {
        self.cancel_map
            .entry(job_id.to_string())
            .or_default()
            .clone()
    }

    pub fn cancel(&self, job_id: &str) {
        self.cancel_map
            .entry(job_id.to_string())
            .or_insert_with(CancellationToken::new)
            .cancel();
    }

    pub fn is_cancelled(&self, job_id: &str) -> bool {
        self.cancel_map
            .get(job_id)
            .is_some_and(|token| token.is_cancelled())
    }

    pub fn cancel_all(&self) {
        for entry in self.cancel_map.iter() {
            entry.cancel();
        }
    }

    pub fn cleanup(&self, job_id: &str) {
        self.cancel_map.remove(job_id);
    }
}

impl Default for JobCanceler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel_before_get_token() {
        let canceler = JobCanceler::new();
        let job_id = "test-job-1";

        canceler.cancel(job_id);

        let token = canceler.get_token(job_id);

        assert!(token.is_cancelled(), "Token should be cancelled");
        assert!(
            canceler.is_cancelled(job_id),
            "is_cancelled should return true"
        );
    }

    #[test]
    fn test_cancel_after_get_token() {
        let canceler = JobCanceler::new();
        let job_id = "test-job-2";

        let token = canceler.get_token(job_id);
        assert!(
            !token.is_cancelled(),
            "Token should not be cancelled initially"
        );

        canceler.cancel(job_id);

        assert!(
            token.is_cancelled(),
            "Token should be cancelled after cancellation"
        );
        assert!(
            canceler.is_cancelled(job_id),
            "is_cancelled should return true"
        );
    }

    #[test]
    fn test_cleanup_removes_token() {
        let canceler = JobCanceler::new();
        let job_id = "test-job-3";

        let token = canceler.get_token(job_id);
        canceler.cancel(job_id);
        assert!(token.is_cancelled());

        canceler.cleanup(job_id);

        assert!(
            !canceler.is_cancelled(job_id),
            "is_cancelled should return false after cleanup"
        );

        let new_token = canceler.get_token(job_id);
        assert!(
            !new_token.is_cancelled(),
            "New token should not be cancelled"
        );
    }

    #[test]
    fn test_cancel_all() {
        let canceler = JobCanceler::new();

        let token1 = canceler.get_token("job-1");
        let token2 = canceler.get_token("job-2");
        let token3 = canceler.get_token("job-3");

        canceler.cancel_all();

        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
        assert!(token3.is_cancelled());
    }

    #[test]
    fn test_cancel_token_before_domain_semaphore_acquire() {
        let canceler = JobCanceler::new();
        let job_id = "test-job-domain-wait";

        let cancel_token = canceler.get_token(job_id);
        assert!(
            !cancel_token.is_cancelled(),
            "Token should not be cancelled initially"
        );

        canceler.cancel(job_id);

        assert!(
            canceler.is_cancelled(job_id),
            "is_cancelled should return true immediately"
        );
        assert!(cancel_token.is_cancelled(), "Token should be cancelled");
    }

    #[test]
    fn test_token_shared_reference() {
        let canceler = JobCanceler::new();
        let job_id = "test-job-4";

        let token1 = canceler.get_token(job_id);
        let token2 = canceler.get_token(job_id);

        canceler.cancel(job_id);
        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
    }
}
