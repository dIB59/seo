use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct JobCanceler {
    cancel_map: Arc<DashMap<String, Arc<AtomicBool>>>,
}

impl JobCanceler {
    pub fn new() -> Self {
        Self {
            cancel_map: Arc::new(DashMap::with_capacity(10)),
        }
    }

    pub fn get_cancel_flag(&self, job_id: &str) -> Arc<AtomicBool> {
        self.cancel_map
            .entry(job_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    /// Mark a job as cancelled. Creates the flag if it doesn't exist yet,
    /// ensuring cancellation works even if called before the job starts processing.
    pub fn set_cancelled(&self, job_id: &str) {
        self.cancel_map
            .entry(job_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(true)))
            .store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self, job_id: &str) -> bool {
        self.cancel_map
            .get(job_id)
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
    }

    pub fn cancel_all(&self) {
        for entry in self.cancel_map.iter_mut() {
            entry.store(true, Ordering::Relaxed);
        }
    }

    /// Clean up the cancel flag for a completed job.
    /// This should be called when a job finishes (either successfully or with error)
    /// to prevent memory leaks from accumulating cancel flags.
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
    fn test_cancel_before_get_flag() {
        // Regression test: cancellation should work even if set_cancelled
        // is called before get_cancel_flag
        let canceler = JobCanceler::new();
        let job_id = "test-job-1";

        // Cancel before getting the flag (simulates cancel before job starts)
        canceler.set_cancelled(job_id);

        // Now get the flag (simulates job starting)
        let flag = canceler.get_cancel_flag(job_id);

        // The flag should be true
        assert!(flag.load(Ordering::Relaxed), "Flag should be true when cancelled before get");
        assert!(canceler.is_cancelled(job_id), "is_cancelled should return true");
    }

    #[test]
    fn test_cancel_after_get_flag() {
        // Normal case: get flag first, then cancel
        let canceler = JobCanceler::new();
        let job_id = "test-job-2";

        // Get flag first (simulates job starting)
        let flag = canceler.get_cancel_flag(job_id);
        assert!(!flag.load(Ordering::Relaxed), "Flag should be false initially");

        // Cancel after getting the flag
        canceler.set_cancelled(job_id);

        // The flag should now be true
        assert!(flag.load(Ordering::Relaxed), "Flag should be true after cancellation");
        assert!(canceler.is_cancelled(job_id), "is_cancelled should return true");
    }

    #[test]
    fn test_cleanup_removes_flag() {
        let canceler = JobCanceler::new();
        let job_id = "test-job-3";

        // Get flag and cancel
        let flag = canceler.get_cancel_flag(job_id);
        canceler.set_cancelled(job_id);
        assert!(flag.load(Ordering::Relaxed));

        // Cleanup
        canceler.cleanup(job_id);

        // Flag should be removed
        assert!(!canceler.is_cancelled(job_id), "is_cancelled should return false after cleanup");

        // Getting a new flag should start fresh (false)
        let new_flag = canceler.get_cancel_flag(job_id);
        assert!(!new_flag.load(Ordering::Relaxed), "New flag should start as false");
    }

    #[test]
    fn test_cancel_all() {
        let canceler = JobCanceler::new();

        // Create flags for multiple jobs
        let flag1 = canceler.get_cancel_flag("job-1");
        let flag2 = canceler.get_cancel_flag("job-2");
        let flag3 = canceler.get_cancel_flag("job-3");

        // Cancel all
        canceler.cancel_all();

        // All flags should be true
        assert!(flag1.load(Ordering::Relaxed));
        assert!(flag2.load(Ordering::Relaxed));
        assert!(flag3.load(Ordering::Relaxed));
    }

    #[test]
    fn test_cancel_flag_before_domain_semaphore_acquire() {
        // Regression test: cancellation should work when job is waiting for domain semaphore
        // This simulates the scenario where:
        // 1. Job A with url example.com is running
        // 2. Job B with url example.com is waiting for domain semaphore
        // 3. Job B gets cancelled while waiting
        // 4. Job B should exit immediately without acquiring the semaphore
        let canceler = JobCanceler::new();
        let job_id = "test-job-domain-wait";

        // Get the cancel flag (simulates job starting and getting flag)
        let cancel_flag = canceler.get_cancel_flag(job_id);
        assert!(!cancel_flag.load(Ordering::Relaxed), "Flag should be false initially");

        // Cancel the job (simulates user cancelling while waiting for domain lock)
        canceler.set_cancelled(job_id);

        // Now check if cancelled - should be true even without acquiring any semaphore
        assert!(canceler.is_cancelled(job_id), "is_cancelled should return true immediately");
        assert!(cancel_flag.load(Ordering::Relaxed), "Flag should be true after cancellation");
    }

    #[test]
    fn test_flag_shared_reference() {
        // Test that the Arc<AtomicBool> is shared correctly
        let canceler = JobCanceler::new();
        let job_id = "test-job-4";

        // Get multiple references to the same flag
        let flag1 = canceler.get_cancel_flag(job_id);
        let flag2 = canceler.get_cancel_flag(job_id);

        // They should be the same underlying AtomicBool
        assert!(
            Arc::ptr_eq(&flag1, &flag2),
            "Multiple calls to get_cancel_flag should return the same Arc"
        );

        // Setting cancelled via canceler should affect both references
        canceler.set_cancelled(job_id);
        assert!(flag1.load(Ordering::Relaxed));
        assert!(flag2.load(Ordering::Relaxed));
    }
}
