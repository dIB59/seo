use crate::domain::Job;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Configuration for the job channel.
#[derive(Debug, Clone)]
pub struct JobChannelConfig {
    /// Maximum number of jobs that can be buffered in the channel.
    pub buffer_size: usize,
}

impl Default for JobChannelConfig {
    fn default() -> Self {
        Self { buffer_size: 100 }
    }
}

/// A wrapper around tokio::sync::mpsc for job distribution.
/// Provides a channel-based mechanism for distributing jobs to workers,
/// replacing the polling-based approach.
pub struct JobChannel {
    /// Sender for dispatching jobs to workers.
    sender: mpsc::Sender<Job>,
    /// Receiver for workers to receive jobs (wrapped in Mutex for sharing).
    receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
}

impl JobChannel {
    /// Create a new job channel with the specified configuration.
    pub fn new(config: JobChannelConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.buffer_size);
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Create a new job channel with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(JobChannelConfig::default())
    }

    /// Get a sender for dispatching jobs.
    pub fn sender(&self) -> mpsc::Sender<Job> {
        self.sender.clone()
    }

    /// Receive the next job from the channel.
    /// Returns None if the channel is closed.
    pub async fn recv(&self) -> Option<Job> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }

    /// Try to receive a job without blocking.
    /// Returns None if no job is immediately available.
    pub async fn try_recv(&self) -> Option<Job> {
        let mut receiver = self.receiver.lock().await;
        receiver.try_recv().ok()
    }

    /// Check if the channel is closed (all senders dropped).
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

/// A notifier for waking up workers when new jobs are available.
/// This is used by the job creation command to signal that a new job
/// needs to be processed.
#[derive(Clone)]
pub struct JobNotifier {
    sender: mpsc::Sender<()>,
}

impl JobNotifier {
    /// Create a new job notifier.
    pub fn new() -> (Self, mpsc::Receiver<()>) {
        let (sender, receiver) = mpsc::channel(1);
        (Self { sender }, receiver)
    }

    /// Notify that a new job is available.
    /// This is a best-effort notification - if the channel is full,
    /// the notification is dropped (which is fine, workers are already
    /// aware they need to check).
    pub async fn notify(&self) {
        // Use try_send to avoid blocking if the channel is full
        let _ = self.sender.try_send(());
    }
}

impl Default for JobNotifier {
    fn default() -> Self {
        Self::new().0
    }
}

/// Combined job channel and notifier for complete job distribution.
/// This provides both the channel for job distribution and the
/// notification mechanism for waking workers.
pub struct JobDispatcher {
    /// Channel for distributing jobs to workers.
    channel: JobChannel,
    /// Notifier for waking workers.
    notifier: JobNotifier,
}

impl JobDispatcher {
    /// Create a new job dispatcher with the specified configuration.
    pub fn new(config: JobChannelConfig) -> Self {
        let channel = JobChannel::new(config);
        let notifier = JobNotifier::new().0;
        Self {
            channel,
            notifier,
        }
    }

    /// Create a new job dispatcher with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(JobChannelConfig::default())
    }

    /// Dispatch a job to workers.
    pub async fn dispatch(&self, job: Job) -> Result<(), mpsc::error::SendError<Job>> {
        self.channel.sender().send(job).await?;
        self.notifier.notify().await;
        Ok(())
    }

    /// Try to dispatch a job without blocking.
    pub fn try_dispatch(&self, job: Job) -> Result<(), Box<mpsc::error::TrySendError<Job>>> {
        self.channel.sender().try_send(job).map_err(Box::new)?;
        let _ = self.notifier.sender.try_send(());
        Ok(())
    }

    /// Get the job channel for workers to receive jobs.
    pub fn channel(&self) -> &JobChannel {
        &self.channel
    }

    /// Get the job notifier for signaling new jobs.
    pub fn notifier(&self) -> &JobNotifier {
        &self.notifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Job, JobSettings};

    fn make_test_job(url: &str) -> Job {
        Job::new(url.to_string(), JobSettings::default())
    }

    #[tokio::test]
    async fn test_job_channel_send_recv() {
        let channel = JobChannel::with_defaults();
        let job = make_test_job("https://example.com");

        // Send job
        channel.sender().send(job.clone()).await.unwrap();

        // Receive job
        let received = channel.recv().await.unwrap();
        assert_eq!(received.url, job.url);
    }

    #[tokio::test]
    async fn test_job_channel_close() {
        let channel = JobChannel::with_defaults();
        
        // Get a sender and drop it - but the channel still has its own sender
        let sender = channel.sender();
        drop(sender);
        
        // The channel is not closed because the internal sender still exists
        // Only when ALL senders are dropped does the channel close
        // This test verifies that behavior
        assert!(!channel.is_closed()); // Still has internal sender
    }

    #[tokio::test]
    async fn test_job_notifier() {
        let (notifier, mut receiver) = JobNotifier::new();

        // Notify
        notifier.notify().await;

        // Should receive notification
        let result = receiver.try_recv();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_job_dispatcher() {
        let dispatcher = JobDispatcher::with_defaults();
        let job = make_test_job("https://example.com");

        // Dispatch job
        dispatcher.dispatch(job.clone()).await.unwrap();

        // Receive job
        let received = dispatcher.channel().recv().await.unwrap();
        assert_eq!(received.url, job.url);
    }
}
