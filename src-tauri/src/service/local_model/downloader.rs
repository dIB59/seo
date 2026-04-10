use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use crate::service::spider::SpiderAgent;

/// Per-model download progress event emitted to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadEvent {
    pub model_id: String,
    pub status: ModelDownloadStatus,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    /// 0.0–1.0, or -1.0 when total size is unknown.
    pub progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ModelDownloadStatus {
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

/// Abstraction over event emission — allows no-op stubs in tests.
pub trait DownloadEmitter: Send + Sync {
    fn emit(&self, event: ModelDownloadEvent);
}

/// Production emitter that forwards to the Tauri app handle.
pub struct TauriDownloadEmitter<R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> TauriDownloadEmitter<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

impl<R: tauri::Runtime> DownloadEmitter for TauriDownloadEmitter<R> {
    fn emit(&self, event: ModelDownloadEvent) {
        use tauri_specta::Event as _;
        if let Err(e) = event.emit(&self.app_handle) {
            tracing::warn!(
                "Failed to emit ModelDownloadEvent for {}: {e}",
                event.model_id
            );
        }
    }
}

/// Manages streaming downloads of GGUF model files with per-model cancellation
/// and resume-on-reconnect support.
///
/// Partial downloads are saved to `<dest>.tmp` and atomically renamed to
/// `<dest>` on completion, so `dest.exists()` is only true for complete files.
pub struct ModelDownloader {
    spider: Arc<dyn SpiderAgent>,
    emitter: Arc<dyn DownloadEmitter>,
    cancellations: Arc<DashMap<String, CancellationToken>>,
}

impl ModelDownloader {
    pub fn new(spider: Arc<dyn SpiderAgent>, emitter: Arc<dyn DownloadEmitter>) -> Self {
        Self {
            spider,
            emitter,
            cancellations: Arc::new(DashMap::new()),
        }
    }

    /// Convenience constructor for production use.
    pub fn with_handle<R: tauri::Runtime + 'static>(app_handle: tauri::AppHandle<R>) -> Result<Self> {
        use crate::service::spider::{ClientType, Spider};
        // Use a no-request-timeout client — model files are several GB and can
        // take many minutes on slow connections.
        let spider = Spider::new_agent(ClientType::Download)?;
        let emitter = Arc::new(TauriDownloadEmitter::new(app_handle));
        Ok(Self::new(spider, emitter))
    }

    /// Start (or resume) a download. Emits `ModelDownloadEvent` while in progress.
    pub async fn download(&self, model_id: &str, url: &str, dest: &Path) -> Result<()> {
        let token = CancellationToken::new();
        self.cancellations.insert(model_id.to_string(), token.clone());

        let result = self.download_inner(model_id, url, dest, token).await;

        self.cancellations.remove(model_id);
        result
    }

    /// Cancel an in-progress download. No-op if the model is not downloading.
    /// The partial `.tmp` file is kept so the download can be resumed later.
    pub fn cancel(&self, model_id: &str) {
        if let Some(token) = self.cancellations.get(model_id) {
            token.cancel();
        }
    }

    // ── Private ──────────────────────────────────────────────────────────────

    async fn download_inner(
        &self,
        model_id: &str,
        url: &str,
        dest: &Path,
        cancel: CancellationToken,
    ) -> Result<()> {
        // Write to a `.tmp` sibling so `dest` only exists when the download is
        // complete (makes `is_downloaded` reliable and enables resume).
        let tmp = dest.with_extension("tmp");

        // How many bytes we already have from a previous (interrupted) attempt.
        let resume_from: u64 = match tokio::fs::metadata(&tmp).await {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        if resume_from > 0 {
            tracing::info!("[downloader] {model_id}: resuming from {resume_from} bytes");
        }

        let mut stream = self.spider.stream_get_range(url, resume_from).await?;

        // If the server ignored the Range header and returned 200, we must
        // discard the partial file and start over.
        let (start_byte, append) = if resume_from > 0 && stream.status == 206 {
            (resume_from, true)
        } else {
            if resume_from > 0 {
                tracing::warn!("[downloader] {model_id}: server returned {} instead of 206; restarting", stream.status);
            }
            (0, false)
        };

        if stream.status != 200 && stream.status != 206 {
            anyhow::bail!("Download failed with HTTP {}", stream.status);
        }

        let total_bytes = stream.content_length.unwrap_or(0);

        if let Some(parent) = tmp.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = if append {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(&tmp)
                .await?
        } else {
            tokio::fs::File::create(&tmp).await?
        };

        let mut downloaded: u64 = start_byte;
        let mut last_reported: u64 = start_byte;
        const REPORT_INTERVAL: u64 = 1_024 * 1_024; // 1 MB

        let result = loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => {
                    // Keep the partial file so the download can be resumed.
                    break Ok(false); // false = cancelled
                }
                chunk = stream.next_chunk() => {
                    match chunk {
                        Err(e) => break Err(e),
                        Ok(None) => break Ok(true), // true = completed normally
                        Ok(Some(chunk)) => {
                            if let Err(e) = file.write_all(&chunk).await {
                                break Err(e.into());
                            }
                            downloaded += chunk.len() as u64;

                            if downloaded.saturating_sub(last_reported) >= REPORT_INTERVAL
                                || (total_bytes > 0 && downloaded >= total_bytes)
                            {
                                self.emit(model_id, ModelDownloadStatus::Downloading, downloaded, total_bytes);
                                last_reported = downloaded;
                            }
                        }
                    }
                }
            }
        };

        // Flush and drop before any rename/cleanup.
        file.flush().await.ok();
        drop(file);

        match result {
            Ok(true) => {
                // Atomic rename: only now does `dest` exist.
                tokio::fs::rename(&tmp, dest).await?;
                self.emit(model_id, ModelDownloadStatus::Completed, downloaded, total_bytes);
                Ok(())
            }
            Ok(false) => {
                // Cancelled — keep the partial .tmp for resume.
                self.emit(model_id, ModelDownloadStatus::Cancelled, downloaded, total_bytes);
                Ok(())
            }
            Err(e) => {
                // Keep the partial .tmp for resume; emit Failed so the UI
                // updates immediately without waiting for the command to return.
                self.emit(model_id, ModelDownloadStatus::Failed, downloaded, total_bytes);
                Err(e)
            }
        }
    }

    fn emit(&self, model_id: &str, status: ModelDownloadStatus, downloaded: u64, total: u64) {
        let progress = compute_progress(downloaded, total);

        self.emitter.emit(ModelDownloadEvent {
            model_id: model_id.to_string(),
            status,
            downloaded_bytes: downloaded,
            total_bytes: total,
            progress,
        });
    }
}

/// Compute the download progress as a fraction in `[0.0, 1.0]`, or
/// `-1.0` when `total` is unknown (Content-Length header missing).
/// Extracted from `ModelDownloader::emit` so it can be unit-tested
/// without spinning up a real downloader.
pub(crate) fn compute_progress(downloaded: u64, total: u64) -> f64 {
    if total > 0 {
        downloaded as f64 / total as f64
    } else {
        -1.0
    }
}

#[cfg(test)]
mod tests {
    //! Characterization tests for `ModelDownloadEvent` and the progress
    //! calculation. Both ship to the frontend via tauri-specta::Event,
    //! so the wire format and the special-case -1.0 sentinel are
    //! observable contracts.

    use super::*;

    // ── compute_progress ─────────────────────────────────────────────────

    #[test]
    fn progress_zero_when_nothing_downloaded() {
        assert_eq!(compute_progress(0, 100), 0.0);
    }

    #[test]
    fn progress_full_when_all_downloaded() {
        assert_eq!(compute_progress(100, 100), 1.0);
    }

    #[test]
    fn progress_half_for_50_percent() {
        assert_eq!(compute_progress(50, 100), 0.5);
    }

    #[test]
    fn progress_returns_negative_one_sentinel_when_total_is_zero() {
        // Pin the unknown-size sentinel: when Content-Length is
        // missing, the downloader passes total=0 and the UI relies on
        // the -1.0 marker to render an indeterminate spinner instead
        // of a 0% bar.
        assert_eq!(compute_progress(123, 0), -1.0);
        assert_eq!(compute_progress(0, 0), -1.0);
    }

    #[test]
    fn progress_handles_downloaded_greater_than_total() {
        // Edge case: server sends more bytes than declared. We don't
        // clamp; the value just goes above 1.0. Pin so a future
        // "clamp to 1.0" change is deliberate.
        assert_eq!(compute_progress(200, 100), 2.0);
    }

    #[test]
    fn progress_handles_large_byte_counts() {
        // Multi-gigabyte models — pin that the f64 division doesn't
        // lose precision in the realistic range.
        let p = compute_progress(2_500_000_000, 5_000_000_000);
        assert!((p - 0.5).abs() < 1e-9);
    }

    // ── ModelDownloadStatus serde ────────────────────────────────────────

    #[test]
    fn status_serializes_with_snake_case() {
        // Wire format pinned for the frontend bindings.
        assert_eq!(
            serde_json::to_string(&ModelDownloadStatus::Downloading).unwrap(),
            "\"downloading\""
        );
        assert_eq!(
            serde_json::to_string(&ModelDownloadStatus::Completed).unwrap(),
            "\"completed\""
        );
        assert_eq!(
            serde_json::to_string(&ModelDownloadStatus::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&ModelDownloadStatus::Cancelled).unwrap(),
            "\"cancelled\""
        );
    }

    #[test]
    fn status_deserializes_from_snake_case() {
        let s: ModelDownloadStatus = serde_json::from_str("\"downloading\"").unwrap();
        assert!(matches!(s, ModelDownloadStatus::Downloading));
    }

    // ── ModelDownloadEvent serde ─────────────────────────────────────────

    #[test]
    fn event_serializes_with_camel_case_field_names() {
        // The struct uses serde(rename_all = "camelCase") because the
        // frontend reads modelId / downloadedBytes / totalBytes /
        // progress / status. Pinning the wire format.
        let event = ModelDownloadEvent {
            model_id: "phi-4".to_string(),
            status: ModelDownloadStatus::Downloading,
            downloaded_bytes: 500,
            total_bytes: 1000,
            progress: 0.5,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["modelId"], "phi-4");
        assert_eq!(json["downloadedBytes"], 500);
        assert_eq!(json["totalBytes"], 1000);
        assert_eq!(json["progress"], 0.5);
        assert_eq!(json["status"], "downloading");
    }

    #[test]
    fn event_round_trips_through_serde() {
        let original = ModelDownloadEvent {
            model_id: "qwen2.5-7b".to_string(),
            status: ModelDownloadStatus::Completed,
            downloaded_bytes: 4_680_000_000,
            total_bytes: 4_680_000_000,
            progress: 1.0,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ModelDownloadEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model_id, original.model_id);
        assert_eq!(parsed.downloaded_bytes, original.downloaded_bytes);
        assert!((parsed.progress - original.progress).abs() < 1e-9);
    }

    // ── DownloadEmitter test stub ────────────────────────────────────────

    /// Capturing emitter useful for any future test that wants to
    /// observe what events the downloader produces.
    #[derive(Default)]
    struct CapturingEmitter {
        events: std::sync::Mutex<Vec<ModelDownloadEvent>>,
    }

    impl DownloadEmitter for CapturingEmitter {
        fn emit(&self, event: ModelDownloadEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    #[test]
    fn capturing_emitter_records_events_in_order() {
        let emitter = CapturingEmitter::default();
        emitter.emit(ModelDownloadEvent {
            model_id: "m1".into(),
            status: ModelDownloadStatus::Downloading,
            downloaded_bytes: 0,
            total_bytes: 100,
            progress: 0.0,
        });
        emitter.emit(ModelDownloadEvent {
            model_id: "m1".into(),
            status: ModelDownloadStatus::Completed,
            downloaded_bytes: 100,
            total_bytes: 100,
            progress: 1.0,
        });
        let events = emitter.events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0].status, ModelDownloadStatus::Downloading));
        assert!(matches!(events[1].status, ModelDownloadStatus::Completed));
    }
}
