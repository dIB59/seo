use super::{
    AuditResult, AuditScores, Auditor, CheckResult, PerformanceMetrics, Score, SeoAuditDetails,
};
use crate::service::spider::SpiderAgent;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

struct PersistentProcess {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

#[derive(Debug, Serialize)]
struct PersistentRequest {
    action: String,
    url: Option<String>,
}

pub struct DeepAuditor {
    sidecar_path: PathBuf,
    persistent_process: Arc<Mutex<Option<PersistentProcess>>>,
    spider: Arc<dyn SpiderAgent>,
}

impl DeepAuditor {
    pub fn new(spider: Arc<dyn SpiderAgent>) -> Self {
        let sidecar_path = Self::find_sidecar_path();
        tracing::info!("[DEEP] Sidecar path: {:?}", sidecar_path);
        Self {
            sidecar_path,
            persistent_process: Arc::new(Mutex::new(None)),
            spider,
        }
    }

    pub fn is_available(&self) -> bool {
        self.sidecar_path.exists()
            || std::process::Command::new(&self.sidecar_path)
                .arg("--help")
                .output()
                .is_ok()
    }

    fn find_sidecar_path() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));

        // Get the target triple suffix for the current platform
        let suffix = Self::get_target_triple();
        let binary_name = format!("lighthouse-runner-{}", suffix);

        // Try production location first (same directory as the main executable)
        let production_path = exe_dir.join(&binary_name);
        if production_path.exists() {
            return production_path;
        }

        // Try without suffix
        let production_path_plain = exe_dir.join("lighthouse-runner");
        if production_path_plain.exists() {
            return production_path_plain;
        }

        #[cfg(target_os = "macos")]
        {
            // Try inside MacOS bundle
            let macos_path = exe_dir.join(&binary_name);
            if macos_path.exists() {
                return macos_path;
            }
        }

        // Try development location
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join("lighthouse-runner")
            .join(&binary_name);
        if dev_path.exists() {
            return dev_path;
        }

        PathBuf::from(binary_name)
    }

    fn get_target_triple() -> &'static str {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            "aarch64-apple-darwin"
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            "x86_64-apple-darwin"
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            "x86_64-unknown-linux-gnu"
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            "x86_64-pc-windows-msvc.exe"
        }
        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64"),
        )))]
        {
            "unknown"
        }
    }

    pub async fn start_persistent(&self) -> Result<()> {
        let mut process = self.persistent_process.lock().await;

        if process.is_some() {
            return Ok(());
        }

        if !self.is_available() {
            anyhow::bail!("Lighthouse sidecar not found at: {:?}", self.sidecar_path);
        }

        tracing::info!("[DEEP] Starting persistent sidecar process...");

        let mut child = Command::new(&self.sidecar_path)
            .arg("--persistent")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn persistent lighthouse-runner")?;

        let stdin = child
            .stdin
            .take()
            .context("Failed to get stdin of sidecar process")?;
        let stdout = child
            .stdout
            .take()
            .context("Failed to get stdout of sidecar process")?;

        let mut stdout = BufReader::new(stdout);

        // Wait for ready signal
        let mut ready_line = String::new();
        stdout
            .read_line(&mut ready_line)
            .await
            .context("Failed to read ready signal from sidecar")?;

        let ready_response: serde_json::Value = serde_json::from_str(&ready_line)
            .context("Failed to parse ready signal from sidecar")?;

        if !ready_response
            .get("ready")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            anyhow::bail!("Sidecar did not report ready status");
        }

        tracing::info!("[DEEP] Persistent process started and ready");

        *process = Some(PersistentProcess {
            child,
            stdin,
            stdout,
        });

        Ok(())
    }

    async fn analyze_persistent(&self, url: &str) -> Result<AuditResult> {
        let mut process_guard = self.persistent_process.lock().await;
        let process = process_guard
            .as_mut()
            .context("Persistent process not started")?;

        let start_time = std::time::Instant::now();
        let request = PersistentRequest {
            action: "analyze".to_string(),
            url: Some(url.to_string()),
        };

        let request_json = serde_json::to_string(&request)?;
        process.stdin.write_all(request_json.as_bytes()).await?;
        process.stdin.write_all(b"\n").await?;
        process.stdin.flush().await?;

        let mut response_line = String::new();
        process
            .stdout
            .read_line(&mut response_line)
            .await
            .context("Failed to read response from persistent sidecar")?;

        let process_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let response: SidecarResponse =
            serde_json::from_str(&response_line).context("Failed to parse sidecar output")?;

        if !response.success {
            anyhow::bail!(
                "Lighthouse failed: {}",
                response.error.unwrap_or_else(|| "Unknown error".into())
            );
        }

        Ok(self.build_result(response, process_time_ms, url).await)
    }

    async fn analyze_oneshot(&self, url: &str) -> Result<AuditResult> {
        let start_time = std::time::Instant::now();

        let output = Command::new(&self.sidecar_path)
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to spawn lighthouse-runner")?;

        let process_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            anyhow::bail!(
                "Lighthouse failed: {}",
                if !stderr.is_empty() {
                    stderr.to_string()
                } else {
                    "Unknown error".into()
                }
            );
        }

        let response: SidecarResponse =
            serde_json::from_str(&stdout).context("Failed to parse lighthouse output")?;

        if !response.success {
            anyhow::bail!(
                "Lighthouse analysis failed: {}",
                response.error.unwrap_or_else(|| "Unknown error".into())
            );
        }

        Ok(self.build_result(response, process_time_ms, url).await)
    }

    async fn build_result(
        &self,
        response: SidecarResponse,
        process_time_ms: f64,
        url: &str,
    ) -> AuditResult {
        let scores = self.convert_scores(&response);

        let load_time_ms = scores
            .performance_metrics
            .as_ref()
            .and_then(|pm| pm.time_to_interactive)
            .or(response.load_time_ms)
            .unwrap_or(process_time_ms);

        let html = match &response.html {
            Some(h) if !h.is_empty() => h.clone(),
            _ => self.fetch_html_fallback(url).await.unwrap_or_default(),
        };

        let content_size = response.content_size.unwrap_or(html.len());
        let status_code = response.status_code.unwrap_or(200);

        AuditResult {
            url: response.url.unwrap_or_else(|| url.to_string()),
            html,
            status_code,
            load_time_ms,
            content_size,
            scores,
        }
    }

    fn convert_scores(&self, response: &SidecarResponse) -> AuditScores {
        let scores = response.scores.as_ref();
        let audits = response.seo_audits.as_ref();
        let perf = response.performance_metrics.as_ref();

        AuditScores {
            performance: scores.and_then(|s| s.performance.map(Score::from)),
            accessibility: scores.and_then(|s| s.accessibility.map(Score::from)),
            best_practices: scores.and_then(|s| s.best_practices.map(Score::from)),
            seo: scores.and_then(|s| s.seo.map(Score::from)),
            seo_details: Self::convert_seo_audits(audits),
            performance_metrics: perf.map(|p| PerformanceMetrics {
                first_contentful_paint: p.first_contentful_paint,
                largest_contentful_paint: p.largest_contentful_paint,
                speed_index: p.speed_index,
                time_to_interactive: p.time_to_interactive,
                total_blocking_time: p.total_blocking_time,
                cumulative_layout_shift: p.cumulative_layout_shift,
            }),
        }
    }

    fn convert_seo_audits(audits: Option<&SidecarSeoAudits>) -> SeoAuditDetails {
        let convert = |audit: Option<&SidecarAudit>| -> CheckResult {
            audit
                .map(|a| CheckResult {
                    passed: a.passed,
                    value: a.value.clone(),
                    score: Score::from(a.score),
                    description: a.description.clone(),
                })
                .unwrap_or_default()
        };

        SeoAuditDetails {
            document_title: convert(audits.and_then(|a| a.document_title.as_ref())),
            meta_description: convert(audits.and_then(|a| a.meta_description.as_ref())),
            viewport: convert(audits.and_then(|a| a.viewport.as_ref())),
            canonical: convert(audits.and_then(|a| a.canonical.as_ref())),
            hreflang: convert(audits.and_then(|a| a.hreflang.as_ref())),
            robots_txt: convert(audits.and_then(|a| a.robots_txt.as_ref())),
            crawlable_anchors: convert(audits.and_then(|a| a.crawlable_anchors.as_ref())),
            link_text: convert(audits.and_then(|a| a.link_text.as_ref())),
            image_alt: convert(audits.and_then(|a| a.image_alt.as_ref())),
            http_status_code: convert(audits.and_then(|a| a.http_status_code.as_ref())),
            is_crawlable: convert(audits.and_then(|a| a.is_crawlable.as_ref())),
        }
    }

    async fn fetch_html_fallback(&self, url: &str) -> Result<String> {
        tracing::warn!("[DEEP] Falling back to direct HTML fetch for URL: {}", url);
        self.spider.fetch_html(url).await
    }
}

#[async_trait]
impl Auditor for DeepAuditor {
    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        tracing::info!("[DEEP] Starting analysis: {}", url);

        // Try persistent process first
        {
            let process = self.persistent_process.lock().await;
            if process.is_some() {
                drop(process);
                return self.analyze_persistent(url).await;
            }
        }

        self.analyze_oneshot(url).await
    }

    fn name(&self) -> &'static str {
        "Deep (Lighthouse)"
    }

    async fn shutdown(&self) -> Result<()> {
        let mut process = self.persistent_process.lock().await;

        if let Some(mut p) = process.take() {
            tracing::info!("[DEEP] Shutting down persistent process...");

            let shutdown_req = PersistentRequest {
                action: "shutdown".to_string(),
                url: None,
            };
            if let Ok(json) = serde_json::to_string(&shutdown_req) {
                let _ = p.stdin.write_all(json.as_bytes()).await;
                let _ = p.stdin.write_all(b"\n").await;
                let _ = p.stdin.flush().await;
            }

            let _ = tokio::time::timeout(std::time::Duration::from_secs(2), p.child.wait()).await;
            let _ = p.child.kill().await;
        }

        Ok(())
    }
}

// ============================================================================
// Sidecar response types (internal)
// ============================================================================

#[derive(Debug, Deserialize)]
struct SidecarResponse {
    success: bool,
    url: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    html: Option<String>,
    #[serde(default)]
    status_code: Option<u16>,
    #[serde(default)]
    content_size: Option<usize>,
    #[serde(default)]
    load_time_ms: Option<f64>,
    #[serde(default)]
    scores: Option<SidecarScores>,
    #[serde(default)]
    seo_audits: Option<SidecarSeoAudits>,
    #[serde(default)]
    performance_metrics: Option<SidecarPerformanceMetrics>,
}

#[derive(Debug, Deserialize, Default)]
struct SidecarScores {
    performance: Option<f64>,
    accessibility: Option<f64>,
    best_practices: Option<f64>,
    seo: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct SidecarSeoAudits {
    document_title: Option<SidecarAudit>,
    meta_description: Option<SidecarAudit>,
    viewport: Option<SidecarAudit>,
    canonical: Option<SidecarAudit>,
    hreflang: Option<SidecarAudit>,
    robots_txt: Option<SidecarAudit>,
    crawlable_anchors: Option<SidecarAudit>,
    link_text: Option<SidecarAudit>,
    image_alt: Option<SidecarAudit>,
    http_status_code: Option<SidecarAudit>,
    is_crawlable: Option<SidecarAudit>,
}

#[derive(Debug, Deserialize, Default)]
struct SidecarAudit {
    passed: bool,
    value: Option<String>,
    score: f64,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct SidecarPerformanceMetrics {
    first_contentful_paint: Option<f64>,
    largest_contentful_paint: Option<f64>,
    speed_index: Option<f64>,
    time_to_interactive: Option<f64>,
    total_blocking_time: Option<f64>,
    cumulative_layout_shift: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_response_parsing() {
        let json = r#"{
            "success": true,
            "url": "https://example.com",
            "scores": {
                "performance": 0.95,
                "seo": 0.90
            }
        }"#;

        let response: SidecarResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.scores.unwrap().performance, Some(0.95));
    }
}
