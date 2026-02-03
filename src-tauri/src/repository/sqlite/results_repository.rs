//! Unified results repository for the redesigned schema.
//!
//! This provides a high-level API to fetch complete job results with
//! all related data in optimized queries.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::models::{
    AiInsight, AnalysisResults, AnalysisSummary, CompleteAnalysisResult, CompleteJobResult,
    Heading, HeadingElement, Image, ImageElement, Issue, IssueSeverity, IssueType, Job,
    JobSettings, JobSummary, LighthouseData, Link, LinkDetail, LinkType, Page, PageAnalysisData,
    SeoIssue,
};
use super::{map_job_status, map_link_type, map_severity};
use url::Url;

pub struct ResultsRepository {
    pool: SqlitePool,
}

impl ResultsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get complete job result with all related data.
    /// This is the main query for displaying analysis results.
    ///
    /// Performance: With the new schema, this uses direct FK lookups:
    /// - 1 query for job
    /// - 1 query for pages (WHERE job_id = ?)
    /// - 1 query for issues (WHERE job_id = ?)
    /// - 1 query for links (WHERE job_id = ?)
    /// - 1 query for lighthouse (JOIN on pages)
    /// - 1 query for AI insights
    ///
    /// Total: 6 simple queries vs the old 5+ queries with expensive JOINs.
    pub async fn get_complete_result(&self, job_id: &str) -> Result<CompleteJobResult> {
        let query_start = std::time::Instant::now();

        // 1. Get job (includes settings and summary)
        let job = self.get_job(job_id).await?;

        // 2. Get pages (direct FK lookup - FAST!)
        let pages = self.get_pages(job_id).await?;
        log::debug!("Fetched {} pages in {:?}", pages.len(), query_start.elapsed());

        // 3. Get issues (direct FK lookup - FAST!)
        let issues = self.get_issues(job_id).await?;
        log::debug!("Fetched {} issues", issues.len());

        // 4. Get links (direct FK lookup - FAST!)
        let links = self.get_links(job_id).await?;
        log::debug!("Fetched {} links", links.len());

        // 5. Get lighthouse data
        let lighthouse = self.get_lighthouse(job_id).await?;
        log::debug!("Fetched {} lighthouse records", lighthouse.len());


        // 6. Get AI insights (optional)
        let ai_insights = self.get_ai_insights(job_id).await.ok();

        let total_time = query_start.elapsed();
        log::info!(
            "Loaded complete result for job {} with {} pages, {} issues, {} links in {:?}",
            job_id,
            pages.len(),
            issues.len(),
            links.len(),
            total_time
        );

        Ok(CompleteJobResult {
            job,
            pages,
            issues,
            links,
            lighthouse,
            ai_insights,
        })
    }

    /// Get complete analysis result (frontend-compatible) with headings/images populated per page.
    pub async fn get_complete_analysis_result(&self, job_id: &str) -> Result<CompleteAnalysisResult> {
        let job = self.get_job(job_id).await?;
        let pages = self.get_pages(job_id).await?;
        let issues = self.get_issues(job_id).await?;
        let links = self.get_links(job_id).await?;
        let lighthouse = self.get_lighthouse(job_id).await?;
        let headings = self.get_headings(job_id).await?;
        let images = self.get_images(job_id).await?;

        let page_url_by_id: std::collections::HashMap<String, String> = pages
            .iter()
            .map(|p| (p.id.clone(), p.url.clone()))
            .collect();

        let mut detailed_links_by_page: std::collections::HashMap<String, Vec<LinkDetail>> =
            std::collections::HashMap::new();
        for link in &links {
            let entry = detailed_links_by_page
                .entry(link.source_page_id.clone())
                .or_default();

            let source_url = page_url_by_id.get(&link.source_page_id);
            let is_external = is_external_by_url(source_url, &link.target_url, &link.link_type);

            entry.push(LinkDetail {
                url: link.target_url.clone(),
                text: link.link_text.clone().unwrap_or_default(),
                is_external,
                is_broken: link.status_code.map(|c| c >= 400).unwrap_or(false),
                status_code: link.status_code,
            });
        }

        let headings_by_page: std::collections::HashMap<String, Vec<HeadingElement>> = headings
            .into_iter()
            .fold(std::collections::HashMap::new(), |mut acc, heading| {
                let tag = format!("h{}", heading.level);
                acc.entry(heading.page_id)
                    .or_insert_with(Vec::new)
                    .push(HeadingElement { tag, text: heading.text });
                acc
            });

        let images_by_page: std::collections::HashMap<String, Vec<ImageElement>> = images
            .into_iter()
            .fold(std::collections::HashMap::new(), |mut acc, image| {
                acc.entry(image.page_id)
                    .or_insert_with(Vec::new)
                    .push(ImageElement { src: image.src, alt: image.alt });
                acc
            });

        let lighthouse_by_page: std::collections::HashMap<String, LighthouseData> = lighthouse
            .into_iter()
            .map(|l| (l.page_id.clone(), l))
            .collect();

        let pages: Vec<PageAnalysisData> = pages
            .into_iter()
            .map(|p| {
                let page_id = p.id.clone();
                let mut data: PageAnalysisData = p.into();

                if let Some(lh) = lighthouse_by_page.get(&page_id) {
                    data.lighthouse_performance = lh.performance_score;
                    data.lighthouse_accessibility = lh.accessibility_score;
                    data.lighthouse_best_practices = lh.best_practices_score;
                    data.lighthouse_seo = lh.seo_score;

                    if let Some(raw) = lh.raw_json.as_deref() {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
                            data.lighthouse_seo_audits = value.get("seo_audits").cloned();
                            data.lighthouse_performance_metrics =
                                value.get("performance_metrics").cloned();
                        }
                    }
                }

                if let Some(links) = detailed_links_by_page.get(&page_id) {
                    data.detailed_links = links.clone();
                    data.links = links.iter().map(|l| l.url.clone()).collect();
                    data.internal_links = links.iter().filter(|l| !l.is_external).count() as i64;
                    data.external_links = links.iter().filter(|l| l.is_external).count() as i64;
                }

                if let Some(headings) = headings_by_page.get(&page_id) {
                    data.h1_count = headings.iter().filter(|h| h.tag == "h1").count() as i64;
                    data.h2_count = headings.iter().filter(|h| h.tag == "h2").count() as i64;
                    data.h3_count = headings.iter().filter(|h| h.tag == "h3").count() as i64;
                    data.headings = headings.clone();
                }

                if let Some(images) = images_by_page.get(&page_id) {
                    data.image_count = images.len() as i64;
                    data.images_without_alt = images
                        .iter()
                        .filter(|img| img.alt.as_deref().unwrap_or("").is_empty())
                        .count() as i64;
                    data.images = images.clone();
                }

                data
            })
            .collect();

        let issues: Vec<SeoIssue> = issues
            .into_iter()
            .map(|issue| {
                let issue_type = map_issue_type(issue.severity);
                let page_id = issue.page_id.clone().unwrap_or_default();
                let page_url = page_url_by_id.get(&page_id).cloned().unwrap_or_default();

                SeoIssue {
                    page_id,
                    issue_type,
                    title: issue.issue_type,
                    description: issue.message,
                    page_url,
                    element: issue.details.clone(),
                    recommendation: issue.details.unwrap_or_default(),
                    line_number: None,
                }
            })
            .collect();

        let analysis = AnalysisResults {
            id: job.id.clone(),
            url: job.url.clone(),
            status: job.status.clone(),
            progress: job.progress,
            total_pages: job.summary.total_pages,
            analyzed_pages: job.summary.pages_crawled,
            started_at: Some(job.created_at),
            completed_at: job.completed_at,
            sitemap_found: false,
            robots_txt_found: false,
            ssl_certificate: job.url.starts_with("https"),
            created_at: job.created_at,
        };

        let summary = AnalysisSummary {
            analysis_id: job.id.clone(),
            seo_score: calculate_seo_score(&job),
            avg_load_time: 0.0,
            total_words: pages.iter().map(|p| p.word_count).sum(),
            total_issues: job.summary.total_issues,
        };

        Ok(CompleteAnalysisResult {
            analysis,
            pages,
            issues,
            summary,
        })
    }

    async fn get_job(&self, job_id: &str) -> Result<Job> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis,
                total_pages, pages_crawled, total_issues, 
                critical_issues, warning_issues, info_issues,
                progress, current_stage, error_message
            FROM jobs
            WHERE id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch job")?;

        Ok(Job {
            id: row.id,
            url: row.url,
            status: map_job_status(row.status.as_str()),
            created_at: parse_datetime(row.created_at.as_str()),
            updated_at: parse_datetime(row.updated_at.as_str()),
            completed_at: row.completed_at.as_deref().map(parse_datetime),
            settings: JobSettings {
                max_pages: row.max_pages,
                include_external_links: false,
                check_images: true,
                mobile_analysis: false,
                lighthouse_analysis: row.lighthouse_analysis != 0,
                delay_between_requests: row.rate_limit_ms,
            },
            summary: JobSummary {
                total_pages: row.total_pages,
                pages_crawled: row.pages_crawled,
                total_issues: row.total_issues,
                critical_issues: row.critical_issues,
                warning_issues: row.warning_issues,
                info_issues: row.info_issues,
            },
            progress: row.progress,
            current_stage: row.current_stage,
            error_message: row.error_message,
        })
    }

    async fn get_pages(&self, job_id: &str) -> Result<Vec<Page>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, url, depth, status_code, content_type,
                title, meta_description, canonical_url, robots_meta,
                word_count, load_time_ms, response_size_bytes, crawled_at
            FROM pages
            WHERE job_id = ?
            ORDER BY depth ASC, url ASC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pages")?;

        Ok(rows
            .into_iter()
            .map(|row| Page {
                id: row.id,
                job_id: row.job_id,
                url: row.url,
                depth: row.depth,
                status_code: row.status_code,
                content_type: row.content_type,
                title: row.title,
                meta_description: row.meta_description,
                canonical_url: row.canonical_url,
                robots_meta: row.robots_meta,
                word_count: row.word_count,
                load_time_ms: row.load_time_ms,
                response_size_bytes: row.response_size_bytes,
                crawled_at: parse_datetime(row.crawled_at.as_str()),
            })
            .collect())
    }

    async fn get_issues(&self, job_id: &str) -> Result<Vec<Issue>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, page_id, type as issue_type, severity, message, details, created_at
            FROM issues
            WHERE job_id = ?
            ORDER BY 
                CASE severity 
                    WHEN 'critical' THEN 1 
                    WHEN 'warning' THEN 2 
                    ELSE 3 
                END,
                type ASC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch issues")?;

        Ok(rows
            .into_iter()
            .map(|row| Issue {
                id: row.id.expect("msg"),
                job_id: row.job_id,
                page_id: row.page_id,
                issue_type: row.issue_type,
                severity: map_severity(row.severity.as_str()),
                message: row.message,
                details: row.details,
                created_at: parse_datetime(row.created_at.as_str()),
            })
            .collect())
    }

    async fn get_links(&self, job_id: &str) -> Result<Vec<Link>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, source_page_id, target_page_id, target_url,
                link_text, link_type, is_followed, status_code
            FROM links
            WHERE job_id = ?
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch links")?;

        Ok(rows
            .into_iter()
            .map(|row| Link {
                id: row.id.expect("Must exist"),
                job_id: row.job_id,
                source_page_id: row.source_page_id,
                target_page_id: row.target_page_id,
                target_url: row.target_url,
                link_text: row.link_text,
                link_type: map_link_type(row.link_type.as_str()),
                is_followed: row.is_followed != 0,
                status_code: row.status_code,
            })
            .collect())
    }

    async fn get_lighthouse(&self, job_id: &str) -> Result<Vec<LighthouseData>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                pl.page_id, pl.performance_score, pl.accessibility_score,
                pl.best_practices_score, pl.seo_score,
                pl.first_contentful_paint_ms, pl.largest_contentful_paint_ms,
                pl.total_blocking_time_ms, pl.cumulative_layout_shift,
                pl.speed_index, pl.time_to_interactive_ms, pl.raw_json
            FROM page_lighthouse pl
            JOIN pages p ON p.id = pl.page_id
            WHERE p.job_id = ?
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch lighthouse data")?;

        Ok(rows
            .into_iter()
            .map(|row| LighthouseData {
                page_id: row.page_id,
                performance_score: row.performance_score,
                accessibility_score: row.accessibility_score,
                best_practices_score: row.best_practices_score,
                seo_score: row.seo_score,
                first_contentful_paint_ms: row.first_contentful_paint_ms,
                largest_contentful_paint_ms: row.largest_contentful_paint_ms,
                total_blocking_time_ms: row.total_blocking_time_ms,
                cumulative_layout_shift: row.cumulative_layout_shift,
                speed_index: row.speed_index,
                time_to_interactive_ms: row.time_to_interactive_ms,
                raw_json: row.raw_json,
            })
            .collect())
    }

    async fn get_headings(&self, job_id: &str) -> Result<Vec<Heading>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                ph.id, ph.page_id, ph.level, ph.text, ph.position
            FROM page_headings ph
            JOIN pages p ON p.id = ph.page_id
            WHERE p.job_id = ?
            ORDER BY ph.page_id, ph.position
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch headings")?;

        Ok(rows
            .into_iter()
            .map(|row| Heading {
                id: row.id.expect("Must exist"),
                page_id: row.page_id,
                level: row.level,
                text: row.text,
                position: row.position,
            })
            .collect())
    }

    async fn get_images(&self, job_id: &str) -> Result<Vec<Image>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                pi.id, pi.page_id, pi.src, pi.alt, pi.width, pi.height,
                pi.loading, pi.is_decorative
            FROM page_images pi
            JOIN pages p ON p.id = pi.page_id
            WHERE p.job_id = ?
            ORDER BY pi.page_id, pi.id
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch images")?;

        Ok(rows
            .into_iter()
            .map(|row| Image {
                id: row.id.expect("Must exist"),
                page_id: row.page_id,
                src: row.src,
                alt: row.alt,
                width: row.width,
                height: row.height,
                loading: row.loading,
                is_decorative: row.is_decorative != 0,
            })
            .collect())
    }

    async fn get_ai_insights(&self, job_id: &str) -> Result<AiInsight> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, job_id, summary, recommendations, raw_response,
                model, created_at, updated_at
            FROM ai_insights
            WHERE job_id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch AI insights")?;

        Ok(AiInsight {
            id: row.id.expect("Must Exist"),
            job_id: row.job_id,
            summary: row.summary,
            recommendations: row.recommendations,
            raw_response: row.raw_response,
            model: row.model,
            created_at: parse_datetime(row.created_at.as_str()),
            updated_at: parse_datetime(row.updated_at.as_str()),
        })
    }

    /// Save AI insights for a job.
    pub async fn save_ai_insights(
        &self,
        job_id: &str,
        summary: Option<&str>,
        recommendations: Option<&str>,
        raw_response: Option<&str>,
        model: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO ai_insights (job_id, summary, recommendations, raw_response, model, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(job_id) DO UPDATE SET
                summary = excluded.summary,
                recommendations = excluded.recommendations,
                raw_response = excluded.raw_response,
                model = excluded.model,
                updated_at = excluded.updated_at
            "#,
            job_id,
            summary,
            recommendations,
            raw_response,
            model,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .context("Failed to save AI insights")?;

        Ok(())
    }
}

/// Parse datetime string to UTC DateTime.
fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn map_issue_type(severity: IssueSeverity) -> IssueType {
    match severity {
        IssueSeverity::Critical => IssueType::Critical,
        IssueSeverity::Warning => IssueType::Warning,
        IssueSeverity::Info => IssueType::Suggestion,
    }
}

fn is_internal_link(link_type: &LinkType) -> bool {
    matches!(link_type, LinkType::Internal)
}

fn is_external_by_url(source_url: Option<&String>, target_url: &str, link_type: &LinkType) -> bool {
    let source_url = match source_url {
        Some(url) => url,
        None => return !is_internal_link(link_type),
    };

    let source = Url::parse(source_url).ok();
    let target = Url::parse(target_url).ok();

    if let (Some(source), Some(target)) = (source, target) {
        let same_host = source.host_str() == target.host_str();
        let same_port = source.port() == target.port();
        return !(same_host && same_port);
    }

    !is_internal_link(link_type)
}

fn calculate_seo_score(job: &Job) -> i64 {
    let total = job.summary.total_issues;
    let critical = job.summary.critical_issues;
    let warning = job.summary.warning_issues;

    if total == 0 {
        return 100;
    }

    let deductions = (critical * 10) + (warning * 5) + (total - critical - warning);
    let score = 100 - deductions;

    score.clamp(0, 100)
}
