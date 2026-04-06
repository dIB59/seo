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
        let progress = if total > 0 {
            downloaded as f64 / total as f64
        } else {
            -1.0
        };

        self.emitter.emit(ModelDownloadEvent {
            model_id: model_id.to_string(),
            status,
            downloaded_bytes: downloaded,
            total_bytes: total,
            progress,
        });
    }
}
