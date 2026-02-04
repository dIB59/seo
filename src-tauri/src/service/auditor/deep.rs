//! Deep Auditor - Full Lighthouse-based SEO analysis.
//!
//! Uses a bundled sidecar binary (lighthouse-runner) to run comprehensive
//! Lighthouse audits. Slower but provides detailed scores and metrics.

use super::{AuditResult, AuditScores, Auditor, CheckResult, PerformanceMetrics, SeoAuditDetails, Score};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Deep auditor using Lighthouse via bundled sidecar binary.
/// 
/// Provides comprehensive SEO analysis including:
/// - Performance scores and Core Web Vitals
/// - Accessibility audit
/// - Best practices check
/// - Detailed SEO audits
/// 
/// Trade-off: ~5-10 seconds per page due to Chrome rendering.
pub struct DeepAuditor {
    sidecar_path: PathBuf,
}

impl DeepAuditor {
    /// Create a new DeepAuditor, locating the sidecar binary.
    pub fn new() -> Self {
        let sidecar_path = Self::find_sidecar_path();
        log::info!("[DEEP] Sidecar path: {:?}", sidecar_path);
        Self { sidecar_path }
    }

    /// Check if the sidecar binary is available.
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
        let suffix = Self::get_target_triple();
        let binary_name = format!("lighthouse-runner-{}", suffix);

        // Try production location
        let production_path = exe_dir.join(&binary_name);
        if production_path.exists() {
            return production_path;
        }

        // Try without suffix
        let plain_path = exe_dir.join("lighthouse-runner");
        if plain_path.exists() {
            return plain_path;
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
        { "aarch64-apple-darwin" }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        { "x86_64-apple-darwin" }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        { "x86_64-unknown-linux-gnu" }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        { "x86_64-pc-windows-msvc.exe" }
        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64"),
        )))]
        { "unknown" }
    }

    fn convert_scores(&self, response: &SidecarResponse) -> AuditScores {
        let scores = response.scores.as_ref();
        let audits = response.seo_audits.as_ref();
        let perf = response.performance_metrics.as_ref();

        AuditScores {
            performance: scores.and_then(|s| s.performance.map(|v| Score::from(v))),
            accessibility: scores.and_then(|s| s.accessibility.map(|v| Score::from(v))),
            best_practices: scores.and_then(|s| s.best_practices.map(|v| Score::from(v))),
            seo: scores.and_then(|s| s.seo.map(|v| Score::from(v))),
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
            audit.map(|a| CheckResult {
                passed: a.passed,
                value: a.value.clone(),
                score: Score::from(a.score),
                description: a.description.clone(),
            }).unwrap_or_default()
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
        log::warn!("[DEEP] Falling back to direct HTML fetch for URL: {}", url);
        let client = rquest::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (compatible; SEOBot/1.0)")
            .send()
            .await?;
        Ok(response.text().await?)
    }
}

impl Default for DeepAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Auditor for DeepAuditor {
    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        log::info!("[DEEP] Starting analysis: {}", url);

        if !self.is_available() {
            anyhow::bail!(
                "Lighthouse sidecar not found at: {:?}",
                self.sidecar_path
            );
        }

        let start_time = std::time::Instant::now();

        let output = Command::new(&self.sidecar_path)
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to spawn lighthouse-runner")?;

        let process_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        log::info!("[DEEP] Process completed in {:.2}ms", process_time_ms);

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stderr.is_empty() {
            log::debug!("[DEEP] stderr: {}", stderr.trim());
        }

        if !output.status.success() {
            anyhow::bail!(
                "Lighthouse failed: {}",
                if !stderr.is_empty() { stderr.to_string() } else { "Unknown error".into() }
            );
        }

        let response: SidecarResponse = serde_json::from_str(&stdout)
            .context("Failed to parse lighthouse output")?;

        if !response.success {
            anyhow::bail!(
                "Lighthouse analysis failed: {}",
                response.error.unwrap_or_else(|| "Unknown error".into())
            );
        }

        let scores = self.convert_scores(&response);

        // Load time priority: TTI -> HTTP time -> process time
        let load_time_ms = scores.performance_metrics.as_ref()
            .and_then(|pm| pm.time_to_interactive)
            .or(response.load_time_ms)
            .unwrap_or(process_time_ms);

        // Get HTML (from Lighthouse or fallback fetch)
        let html = match &response.html {
            Some(h) if !h.is_empty() => h.clone(),
            _ => self.fetch_html_fallback(url).await.unwrap_or_default(),
        };

        let content_size = response.content_size.unwrap_or(html.len());
        let status_code = response.status_code.unwrap_or(200);

        log::info!(
            "[DEEP] Complete - status: {}, size: {} bytes, load: {:.2}ms",
            status_code, content_size, load_time_ms
        );

        Ok(AuditResult {
            url: response.url.unwrap_or_else(|| url.to_string()),
            html,
            status_code,
            load_time_ms,
            content_size,
            scores,
        })
    }

    fn name(&self) -> &'static str {
        "Deep (Lighthouse)"
    }

    async fn shutdown(&self) -> Result<()> {
        log::info!("[DEEP] Shutdown (no-op for sidecar approach)");
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
