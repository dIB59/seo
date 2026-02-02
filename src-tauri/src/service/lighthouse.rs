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

/// Result of an individual audit check
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditResult {
    pub passed: bool,
    pub value: Option<String>,
    pub score: f64,
    #[serde(default)]
    pub description: Option<String>,
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
        log::trace!("[LIGHTHOUSE-SIDECAR] Executing command: {:?} {}", self.sidecar_path, url);
        let output = Command::new(&self.sidecar_path)
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to spawn lighthouse-runner sidecar")?;
        
        let load_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        log::info!("[LIGHTHOUSE-SIDECAR] Process completed in {:.2}ms", load_time_ms);
        
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
            "[LIGHTHOUSE-SIDECAR] Analysis complete - status: {}, content: {} bytes, time: {:.2}ms",
            status_code, content_size, load_time_ms
        );
        
        Ok(PageFetchResult {
            url: response.url.unwrap_or_else(|| url.to_string()),
            html,
            status_code,
            load_time_ms,
            content_size,
            scores,
        })
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
            seo_audits: self.convert_seo_audits(sidecar_audits),
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
    
    fn convert_seo_audits(&self, audits: Option<&SidecarSeoAudits>) -> SeoAuditDetails {
        let audits = match audits {
            Some(a) => a,
            None => return SeoAuditDetails::default(),
        };
        
        fn convert_audit(audit: Option<&SidecarAudit>) -> AuditResult {
            audit.map(|a| AuditResult {
                passed: a.passed,
                value: a.value.clone(),
                score: a.score,
                description: a.description.clone(),
            }).unwrap_or_default()
        }
        
        SeoAuditDetails {
            document_title: convert_audit(audits.document_title.as_ref()),
            meta_description: convert_audit(audits.meta_description.as_ref()),
            viewport: convert_audit(audits.viewport.as_ref()),
            canonical: convert_audit(audits.canonical.as_ref()),
            hreflang: convert_audit(audits.hreflang.as_ref()),
            robots_txt: convert_audit(audits.robots_txt.as_ref()),
            crawlable_anchors: convert_audit(audits.crawlable_anchors.as_ref()),
            link_text: convert_audit(audits.link_text.as_ref()),
            image_alt: convert_audit(audits.image_alt.as_ref()),
            http_status_code: convert_audit(audits.http_status_code.as_ref()),
            is_crawlable: convert_audit(audits.is_crawlable.as_ref()),
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
