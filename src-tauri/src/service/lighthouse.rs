//! Lighthouse service using a bundled sidecar binary for real Lighthouse audits.
//!
//! This module spawns the \`lighthouse-runner\` sidecar which is a standalone Node.js
//! executable that runs actual Lighthouse audits and returns JSON results.

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

/// Service for running Lighthouse audits via Node.js
pub struct LighthouseService {
    script_path: PathBuf,
    node_path: String,
}

impl LighthouseService {
    /// Create a new LighthouseService, locating the script and Node.js
    pub fn new() -> Self {
        let script_path = Self::find_script_path();
        let node_path = Self::find_node_path();
        log::info!("Lighthouse script path: {:?}", script_path);
        log::info!("Node.js path: {}", node_path);
        Self { script_path, node_path }
    }

    /// Find Node.js executable
    fn find_node_path() -> String {
        // Try common locations
        let candidates = if cfg!(target_os = "windows") {
            vec![
                "node.exe",
                "C:\\Program Files\\nodejs\\node.exe",
                "C:\\Program Files (x86)\\nodejs\\node.exe",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "node",
                "/usr/local/bin/node",
                "/opt/homebrew/bin/node",
                "/usr/bin/node",
            ]
        } else {
            vec![
                "node",
                "/usr/bin/node",
                "/usr/local/bin/node",
            ]
        };

        for candidate in &candidates {
            if std::process::Command::new(candidate)
                .arg("--version")
                .output()
                .is_ok()
            {
                return candidate.to_string();
            }
        }

        // Fallback - assume node is in PATH
        "node".to_string()
    }

    /// Find the path to the lighthouse-runner script
    fn find_script_path() -> PathBuf {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
        
        // Try production location first (Resources/lighthouse-runner/index.js on macOS)
        #[cfg(target_os = "macos")]
        {
            let resources_path = exe_dir
                .parent()
                .map(|p| p.join("Resources").join("lighthouse-runner").join("index.js"));
            if let Some(path) = resources_path {
                if path.exists() {
                    return path;
                }
            }
        }
        
        // Try next to the binary
        let production_path = exe_dir.join("lighthouse-runner").join("index.js");
        if production_path.exists() {
            return production_path;
        }
        
        // Try development location
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join("lighthouse-runner")
            .join("index.js");
        if dev_path.exists() {
            return dev_path;
        }
        
        // Fallback
        PathBuf::from("lighthouse-runner/index.js")
    }

    /// Check if Node.js is available
    pub fn is_available(&self) -> bool {
        std::process::Command::new(&self.node_path)
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Analyze a URL using Lighthouse via Node.js
    pub async fn analyze(&self, url: &str) -> Result<PageFetchResult> {
        log::info!("Running Lighthouse analysis for: {}", url);
        
        if !self.is_available() {
            anyhow::bail!("Node.js is not available. Please install Node.js to use Lighthouse analysis.");
        }
        
        let start_time = std::time::Instant::now();
        
        // Spawn node with the script
        let output = Command::new(&self.node_path)
            .arg(&self.script_path)
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to spawn Node.js for Lighthouse analysis")?;
        
        let load_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        
        // Parse stdout as JSON
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !output.status.success() {
            log::error!("Lighthouse sidecar failed: {}", stderr);
            anyhow::bail!(
                "Lighthouse analysis failed: {}",
                if !stderr.is_empty() { stderr.to_string() } else { "Unknown error".to_string() }
            );
        }
        
        let response: SidecarResponse = serde_json::from_str(&stdout)
            .context("Failed to parse lighthouse-runner output")?;
        
        if !response.success {
            anyhow::bail!(
                "Lighthouse analysis failed: {}",
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            );
        }
        
        // Convert sidecar response to our types
        let scores = self.convert_scores(&response);
        
        // Fetch HTML separately since Lighthouse doesn't give us the raw HTML
        let html = self.fetch_html(url).await.unwrap_or_default();
        let content_size = html.len();
        
        Ok(PageFetchResult {
            url: response.url.unwrap_or_else(|| url.to_string()),
            html,
            status_code: 200,
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
