//! Lighthouse service using a bundled sidecar binary for real Lighthouse audits.
//!
//! This module spawns the `lighthouse-runner` sidecar which is a standalone executable
//! (bundled Node.js + Lighthouse) that runs actual Lighthouse audits and returns JSON results.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Result from a Lighthouse analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageFetchResult {
    pub url: String,
    pub html: String,
    pub status_code: u16,
    pub load_time_ms: f64,
    pub content_size: usize,
    pub scores: LighthouseScores,
}

/// Lighthouse category scores (0.0 to 1.0)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LighthouseScores {
    pub performance: Option<f64>,
    pub accessibility: Option<f64>,
    pub best_practices: Option<f64>,
    pub seo: Option<f64>,
    pub seo_audits: SeoAuditDetails,
    pub performance_metrics: Option<PerformanceMetrics>,
}

// Conversion from new AuditScores to legacy LighthouseScores
impl From<crate::service::auditor::AuditScores> for LighthouseScores {
    fn from(scores: crate::service::auditor::AuditScores) -> Self {
        Self {
            performance: scores.performance,
            accessibility: scores.accessibility,
            best_practices: scores.best_practices,
            seo: scores.seo,
            seo_audits: scores.seo_details.into(),
            performance_metrics: scores.performance_metrics.map(|pm| pm.into()),
        }
    }
}

/// Detailed performance metrics from Lighthouse
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub first_contentful_paint: Option<f64>,
    pub largest_contentful_paint: Option<f64>,
    pub speed_index: Option<f64>,
    pub time_to_interactive: Option<f64>,
    pub total_blocking_time: Option<f64>,
    pub cumulative_layout_shift: Option<f64>,
}

impl From<crate::service::auditor::PerformanceMetrics> for PerformanceMetrics {
    fn from(pm: crate::service::auditor::PerformanceMetrics) -> Self {
        Self {
            first_contentful_paint: pm.first_contentful_paint,
            largest_contentful_paint: pm.largest_contentful_paint,
            speed_index: pm.speed_index,
            time_to_interactive: pm.time_to_interactive,
            total_blocking_time: pm.total_blocking_time,
            cumulative_layout_shift: pm.cumulative_layout_shift,
        }
    }
}

/// Result of an individual audit check
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditResult {
    pub passed: bool,
    pub value: Option<String>,
    pub score: f64,
    #[serde(default)]
    pub description: Option<String>,
}

impl From<crate::service::auditor::CheckResult> for AuditResult {
    fn from(cr: crate::service::auditor::CheckResult) -> Self {
        Self {
            passed: cr.passed,
            value: cr.value,
            score: cr.score,
            description: cr.description,
        }
    }
}

/// Detailed SEO audit results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeoAuditDetails {
    pub document_title: AuditResult,
    pub meta_description: AuditResult,
    pub viewport: AuditResult,
    pub canonical: AuditResult,
    pub hreflang: AuditResult,
    pub robots_txt: AuditResult,
    pub crawlable_anchors: AuditResult,
    pub link_text: AuditResult,
    pub image_alt: AuditResult,
    pub http_status_code: AuditResult,
    pub is_crawlable: AuditResult,
}

impl From<crate::service::auditor::SeoAuditDetails> for SeoAuditDetails {
    fn from(details: crate::service::auditor::SeoAuditDetails) -> Self {
        Self {
            document_title: details.document_title.into(),
            meta_description: details.meta_description.into(),
            viewport: details.viewport.into(),
            canonical: details.canonical.into(),
            hreflang: details.hreflang.into(),
            robots_txt: details.robots_txt.into(),
            crawlable_anchors: details.crawlable_anchors.into(),
            link_text: details.link_text.into(),
            image_alt: details.image_alt.into(),
            http_status_code: details.http_status_code.into(),
            is_crawlable: details.is_crawlable.into(),
        }
    }
}

/// Request to analyze a URL with Lighthouse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighthouseRequest {
    pub url: String,
    pub job_id: i64,
    pub request_id: String,
}

/// Raw response from the lighthouse-runner sidecar
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
    load_time_ms: Option<f64>,  // Actual HTTP response time
    #[serde(default)]
    scores: Option<SidecarScores>,
    #[serde(default)]
    seo_audits: Option<SidecarSeoAudits>,
    #[serde(default)]
    performance_metrics: Option<SidecarPerformanceMetrics>,
    // Batch mode fields
    #[serde(default)]
    #[allow(dead_code)] // Used for JSON deserialization, indicates batch response
    batch: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    results: Option<Vec<SidecarResponse>>,
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

/// Service for running Lighthouse audits via bundled sidecar binary
pub struct LighthouseService {
    sidecar_path: PathBuf,
}

impl LighthouseService {
    /// Create a new LighthouseService, locating the sidecar binary
    pub fn new() -> Self {
        let sidecar_path = Self::find_sidecar_path();
        log::info!("Lighthouse sidecar path: {:?}", sidecar_path);
        Self { sidecar_path }
    }

    /// Find the path to the lighthouse-runner sidecar binary
    fn find_sidecar_path() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
        
        // Get the target triple suffix for the current platform
        let suffix = Self::get_target_triple();
        let binary_name = format!("lighthouse-runner-{}", suffix);
        
        // On macOS, also try without suffix (Tauri adds it at runtime)
        let binary_name_plain = "lighthouse-runner";
        
        // Try production location first (same directory as the main executable)
        let production_path = exe_dir.join(&binary_name);
        if production_path.exists() {
            return production_path;
        }
        
        // Try without suffix
        let production_path_plain = exe_dir.join(binary_name_plain);
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
        
        // Try development location (in binaries/lighthouse-runner directory)
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join("lighthouse-runner")
            .join(&binary_name);
        if dev_path.exists() {
            return dev_path;
        }
        
        // Fallback - just use the binary name and hope it's in PATH or current dir
        PathBuf::from(binary_name)
    }
    
    /// Get the target triple suffix for the current platform
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

    /// Check if the sidecar binary is available
    pub fn is_available(&self) -> bool {
        self.sidecar_path.exists() || {
            // Also check if we can run it (in case it's in PATH)
            std::process::Command::new(&self.sidecar_path)
                .arg("--help")
                .output()
                .is_ok()
        }
    }

    /// Analyze a URL using Lighthouse via the bundled sidecar binary
    pub async fn analyze(&self, url: &str) -> Result<PageFetchResult> {
        log::info!("[LIGHTHOUSE-SIDECAR] Starting analysis for: {}", url);
        log::debug!("[LIGHTHOUSE-SIDECAR] Sidecar binary path: {:?}", self.sidecar_path);
        
        if !self.is_available() {
            log::error!("[LIGHTHOUSE-SIDECAR] Binary not found at: {:?}", self.sidecar_path);
            anyhow::bail!(
                "Lighthouse sidecar binary not found at: {:?}. This is a packaging error.",
                self.sidecar_path
            );
        }
        log::debug!("[LIGHTHOUSE-SIDECAR] Binary verified, spawning process...");
        
        let start_time = std::time::Instant::now();
        
        // Spawn the sidecar binary directly
        log::debug!("[LIGHTHOUSE-SIDECAR] Executing command: {:?} {}", self.sidecar_path, url);
        let output = Command::new(&self.sidecar_path)
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to spawn lighthouse-runner sidecar")?;
        
        let process_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        log::info!("[LIGHTHOUSE-SIDECAR] Process completed in {:.2}ms", process_time_ms);
        
        // Parse stdout as JSON
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        log::trace!("[LIGHTHOUSE-SIDECAR] stdout length: {} bytes", stdout.len());
        if !stderr.is_empty() {
            log::debug!("[LIGHTHOUSE-SIDECAR] stderr: {}", stderr.trim());
        }
        
        if !output.status.success() {
            log::error!("[LIGHTHOUSE-SIDECAR] Process failed with status: {:?}", output.status);
            log::error!("[LIGHTHOUSE-SIDECAR] stderr: {}", stderr);
            anyhow::bail!(
                "Lighthouse analysis failed: {}",
                if !stderr.is_empty() { stderr.to_string() } else { "Unknown error".to_string() }
            );
        }
        
        log::debug!("[LIGHTHOUSE-SIDECAR] Parsing JSON response...");
        let response: SidecarResponse = serde_json::from_str(&stdout)
            .context("Failed to parse lighthouse-runner output")?;
        
        if !response.success {
            let error_msg = response.error.unwrap_or_else(|| "Unknown error".to_string());
            log::error!("[LIGHTHOUSE-SIDECAR] Analysis reported failure: {}", error_msg);
            anyhow::bail!("Lighthouse analysis failed: {}", error_msg);
        }
        
        log::debug!("[LIGHTHOUSE-SIDECAR] Converting scores...");
        // Convert sidecar response to our types
        let scores = self.convert_scores(&response);
        log::debug!(
            "[LIGHTHOUSE-SIDECAR] Scores - perf: {:?}, access: {:?}, seo: {:?}, best-practices: {:?}",
            scores.performance, scores.accessibility, scores.seo, scores.best_practices
        );
        
        // Use actual page load time:
        // 1. First try TTI (Time to Interactive) from Lighthouse performance metrics
        // 2. Then try load_time_ms from sidecar (actual HTTP response time)
        // 3. Fallback to process time (includes Chrome startup, not ideal but better than nothing)
        let actual_load_time_ms = scores.performance_metrics.as_ref()
            .and_then(|pm| pm.time_to_interactive)
            .or(response.load_time_ms)
            .unwrap_or(process_time_ms);
        
        log::info!(
            "[LIGHTHOUSE-SIDECAR] Load time: {:.2}ms, Process time: {:.2}ms",
            actual_load_time_ms, process_time_ms
        );
        
        // Use rendered HTML from Lighthouse (JS-executed content)
        // Falls back to fetching if Lighthouse didn't return HTML
        let html = if let Some(ref h) = response.html {
            if !h.is_empty() {
                log::debug!("[LIGHTHOUSE-SIDECAR] Using rendered HTML from Lighthouse ({} bytes)", h.len());
                h.clone()
            } else {
                log::warn!("[LIGHTHOUSE-SIDECAR] Lighthouse returned empty HTML, fetching separately");
                self.fetch_html(url).await.unwrap_or_default()
            }
        } else {
            log::warn!("[LIGHTHOUSE-SIDECAR] Lighthouse didn't return HTML, fetching separately");
            self.fetch_html(url).await.unwrap_or_default()
        };
        
        let content_size = response.content_size.unwrap_or(html.len());
        let status_code = response.status_code.unwrap_or(200);
        
        log::info!(
            "[LIGHTHOUSE-SIDECAR] Analysis complete - status: {}, content: {} bytes, page_load: {:.2}ms, process: {:.2}ms",
            status_code, content_size, actual_load_time_ms, process_time_ms
        );
        
        Ok(PageFetchResult {
            url: response.url.unwrap_or_else(|| url.to_string()),
            html,
            status_code,
            load_time_ms: actual_load_time_ms,
            content_size,
            scores,
        })
    }
    
    /// Analyze multiple URLs sequentially.
    /// Each URL spawns a new sidecar process with its own Chrome instance.
    pub async fn analyze_urls(&self, urls: &[String]) -> Vec<Result<PageFetchResult>> {
        if urls.is_empty() {
            return Vec::new();
        }
        
        log::info!("[LIGHTHOUSE-SIDECAR] Analyzing {} URLs sequentially", urls.len());
        
        let mut results = Vec::with_capacity(urls.len());
        
        for (i, url) in urls.iter().enumerate() {
            log::debug!("[LIGHTHOUSE-SIDECAR] Analyzing {}/{}: {}", i + 1, urls.len(), url);
            let result = self.analyze(url).await;
            results.push(result);
        }
        
        log::info!(
            "[LIGHTHOUSE-SIDECAR] Analysis complete: {}/{} successful",
            results.iter().filter(|r| r.is_ok()).count(),
            results.len()
        );
        
        results
    }
    

    
    /// Helper to convert SEO audits from sidecar response
    fn convert_seo_audits_from_option(audits: Option<&SidecarSeoAudits>) -> SeoAuditDetails {
        let default_audit = || AuditResult {
            passed: false,
            value: None,
            score: 0.0,
            description: None,
        };
        
        let convert = |audit: Option<&SidecarAudit>| -> AuditResult {
            match audit {
                Some(a) => AuditResult {
                    passed: a.passed,
                    value: a.value.clone(),
                    score: a.score,
                    description: a.description.clone(),
                },
                None => default_audit(),
            }
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
    
    fn convert_scores(&self, response: &SidecarResponse) -> LighthouseScores {
        let sidecar_scores = response.scores.as_ref();
        let sidecar_audits = response.seo_audits.as_ref();
        let sidecar_perf = response.performance_metrics.as_ref();
        
        LighthouseScores {
            performance: sidecar_scores.and_then(|s| s.performance),
            accessibility: sidecar_scores.and_then(|s| s.accessibility),
            best_practices: sidecar_scores.and_then(|s| s.best_practices),
            seo: sidecar_scores.and_then(|s| s.seo),
            seo_audits: Self::convert_seo_audits_from_option(sidecar_audits),
            performance_metrics: sidecar_perf.map(|p| PerformanceMetrics {
                first_contentful_paint: p.first_contentful_paint,
                largest_contentful_paint: p.largest_contentful_paint,
                speed_index: p.speed_index,
                time_to_interactive: p.time_to_interactive,
                total_blocking_time: p.total_blocking_time,
                cumulative_layout_shift: p.cumulative_layout_shift,
            }),
        }
    }
    
    async fn fetch_html(&self, url: &str) -> Result<String> {
        let client = rquest::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (compatible; SEOBot/1.0)")
            .send()
            .await?;
        Ok(response.text().await?)
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Lighthouse service shutdown (no-op for sidecar approach)");
        Ok(())
    }
}

impl Default for LighthouseService {
    fn default() -> Self {
        Self::new()
    }
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
                "accessibility": 0.87,
                "best_practices": 0.92,
                "seo": 0.90
            },
            "seo_audits": {
                "document_title": { "passed": true, "value": "Example", "score": 1.0 },
                "meta_description": { "passed": false, "value": null, "score": 0.0 }
            },
            "performance_metrics": {
                "first_contentful_paint": 1200.0,
                "largest_contentful_paint": 2500.0
            }
        }"#;
        
        let response: SidecarResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.scores.unwrap().performance, Some(0.95));
    }
    
    #[test]
    fn test_sidecar_error_response_parsing() {
        let json = r#"{
            "success": false,
            "url": "https://invalid.example",
            "error": "Connection refused"
        }"#;
        
        let response: SidecarResponse = serde_json::from_str(json).unwrap();
        assert!(!response.success);
        assert_eq!(response.error, Some("Connection refused".to_string()));
    }
}
