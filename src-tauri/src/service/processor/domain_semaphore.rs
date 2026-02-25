use crate::domain::extract_root_domain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;

/// Manages per-domain semaphores to ensure jobs for the same domain
/// (including subdomains) don't run concurrently due to rate limiting.
pub struct DomainSemaphore {
    /// Map of root domain -> semaphore (max 1 permit per domain)
    domains: Mutex<HashMap<String, Arc<Semaphore>>>,
}

impl DomainSemaphore {
    /// Create a new domain semaphore.
    pub fn new() -> Self {
        Self {
            domains: Mutex::new(HashMap::new()),
        }
    }

    /// Acquire a permit for the given domain.
    /// This will block if another job for the same domain is running.
    /// If a cancel_flag is provided, it will be checked periodically to allow
    /// early exit if the job is cancelled while waiting for the domain lock.
    pub async fn acquire(&self, url_str: &str) -> Option<DomainPermit> {
        self.acquire_with_cancel(url_str, None).await
    }

    /// Acquire a permit for the given domain with a cancel token.
    /// The cancel token allows immediate cancellation while waiting for
    /// the domain semaphore using tokio::select!.
    pub async fn acquire_with_cancel(
        &self,
        url_str: &str,
        cancel_token: Option<&CancellationToken>,
    ) -> Option<DomainPermit> {
        let root_domain = extract_root_domain(url_str)?;
        
        let semaphore = {
            let mut domains = self.domains.lock().await;
            domains
                .entry(root_domain.clone())
                .or_insert_with(|| Arc::new(Semaphore::new(1)))
                .clone()
        };
        
        tracing::debug!(
            "Waiting for domain lock: {} (from {})",
            root_domain,
            url_str
        );
        
        if let Some(token) = cancel_token {
            if token.is_cancelled() {
                tracing::debug!("Job cancelled before acquiring domain lock");
                return None;
            }
        }
        
        let permit = if let Some(token) = cancel_token {
            tokio::select! {
                permit = Arc::clone(&semaphore).acquire_owned() => {
                    permit.ok()?
                }
                _ = token.cancelled() => {
                    tracing::debug!("Job cancelled while waiting for domain lock");
                    return None;
                }
            }
        } else {
            Arc::clone(&semaphore).acquire_owned().await.ok()?
        };
        
        tracing::info!("Acquired domain lock: {}", root_domain);
        
        Some(DomainPermit {
            permit,
            domain: root_domain,
        })
    }
}

impl Default for DomainSemaphore {
    fn default() -> Self {
        Self::new()
    }
}

/// A permit that holds the domain lock.
/// When dropped, the lock is released.
pub struct DomainPermit {
    #[allow(dead_code)]
    permit: tokio::sync::OwnedSemaphorePermit,
    domain: String,
}

impl DomainPermit {
    /// Get the domain this permit is for.
    pub fn domain(&self) -> &str {
        &self.domain
    }
}

impl Drop for DomainPermit {
    fn drop(&mut self) {
        tracing::info!("Released domain lock: {}", self.domain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_domain_semaphore_blocks_same_domain() {
        let semaphore = Arc::new(DomainSemaphore::new());
        let semaphore_clone = semaphore.clone();
        
        // Acquire lock for example.com
        let permit1 = semaphore.acquire("https://example.com").await;
        assert!(permit1.is_some());
        
        // Try to acquire again - this should block, so we use a timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            semaphore_clone.acquire("https://blog.example.com")
        ).await;
        
        // Should timeout because the domain is locked
        assert!(result.is_err(), "Should have timed out waiting for domain lock");
        
        // Drop the permit
        drop(permit1);
        
        // Now we should be able to acquire
        let permit2 = semaphore.acquire("https://blog.example.com").await;
        assert!(permit2.is_some());
    }

    #[tokio::test]
    async fn test_domain_semaphore_allows_different_domains() {
        let semaphore = Arc::new(DomainSemaphore::new());
        
        // Acquire lock for example.com
        let permit1 = semaphore.acquire("https://example.com").await;
        assert!(permit1.is_some());
        
        // Should be able to acquire lock for different domain immediately
        let permit2 = semaphore.acquire("https://different.com").await;
        assert!(permit2.is_some());
        
        // Both permits should be held
        drop(permit1);
        drop(permit2);
    }

    /// Regression test: cancellation should be checked before domain semaphore acquisition
    /// 
    /// This tests the scenario where:
    /// 1. Job A with url example.com is running and holds the domain semaphore
    /// 2. Job B with url example.com is waiting for the domain semaphore
    /// 3. Job B is cancelled while waiting
    /// 4. Job B should detect the cancellation and exit immediately
    ///
    /// The fix ensures we use tokio::select! with CancellationToken for immediate cancellation.
    #[tokio::test]
    async fn test_cancel_while_waiting_for_domain_semaphore() {
        let semaphore = Arc::new(DomainSemaphore::new());
        let cancel_token = CancellationToken::new();
        
        // Acquire lock for example.com (simulates Job A running)
        let permit1 = semaphore.acquire("https://example.com").await;
        assert!(permit1.is_some(), "First job should acquire domain lock");
        
        let cancel_token_clone = cancel_token.clone();
        let semaphore_clone = semaphore.clone();
        
        // Spawn a task that tries to acquire the same domain (simulates Job B waiting)
        let acquire_task = tokio::spawn(async move {
            semaphore_clone.acquire_with_cancel("https://example.com", Some(&cancel_token_clone)).await
        });
        
        // Give the task time to start waiting
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        // Cancel the second job
        cancel_token.cancel();
        
        // The task should exit immediately after seeing the cancel token
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            acquire_task
        ).await;
        
        // The task should complete (not timeout) because it detected cancellation
        assert!(result.is_ok(), "Task should complete after detecting cancellation");
        
        let permit_opt = result.unwrap().unwrap();
        // The permit should be None because the job was cancelled before acquiring
        assert!(permit_opt.is_none(), "Cancelled job should not acquire domain lock");
        
        drop(permit1);
    }
}
