use std::collections::HashMap;

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use url::Url;
use uuid::Uuid;

use crate::{
    domain::models::{
        AnalysisResults, AnalysisStatus, AnalysisSummary, CompleteAnalysisResult, HeadingElement,
        ImageElement, LinkElement, PageAnalysisData, SeoIssue,
    },
    repository::sqlite::{map_issue_type, map_job_status},
    service::job_processor::PageEdge,
};

pub struct ResultsRepository {
    pool: SqlitePool,
}

impl ResultsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        url: &str,
        sitemap: bool,
        robots: bool,
        ssl: bool,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO analysis_results \
             (id, url, status, progress, analyzed_pages, total_pages, started_at, \
              sitemap_found, robots_txt_found, ssl_certificate) \
             VALUES (?, ?, 'analyzing', 0, 0, 0, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(url)
        .bind(now)
        .bind(sitemap)
        .bind(robots)
        .bind(ssl)
        .execute(&self.pool)
        .await
        .context("Failed to create analysis result")?;

        Ok(id)
    }

    pub async fn update_progress(
        &self,
        id: &str,
        progress: f64,
        analyzed: i64,
        total: i64,
    ) -> Result<()> {
        sqlx::query("UPDATE analysis_results SET progress = ?, analyzed_pages = ?, total_pages = ? WHERE id = ?")
            .bind(progress)
            .bind(analyzed)
            .bind(total)
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to update progress")?;
        Ok(())
    }

    //TODO:
    //Drop coloum of analysis results
    pub async fn finalize(&self, id: &str, status: AnalysisStatus) -> Result<()> {
        sqlx::query("UPDATE analysis_results SET status = ?, completed_at = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to finalize analysis")?;
        Ok(())
    }

    pub async fn get_result_by_job_id(&self, job_id: i64) -> Result<CompleteAnalysisResult> {
        let analysis_result_row = sqlx::query!(
            r#"
            SELECT ar.id as "id!" , ar.url, ar.status, ar.progress, ar.analyzed_pages, ar.total_pages,
                   ar.started_at, ar.created_at, ar.completed_at, ar.sitemap_found, ar.robots_txt_found, ar.ssl_certificate
            FROM analysis_results ar
            JOIN analysis_jobs aj ON aj.result_id = ar.id
            WHERE aj.id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch analysis result by job ID")?;

        let analysis: AnalysisResults = AnalysisResults {
            id: analysis_result_row.id.clone(),
            url: analysis_result_row.url.clone(),
            status: map_job_status(&analysis_result_row.status),
            progress: analysis_result_row.progress,
            analyzed_pages: analysis_result_row.analyzed_pages,
            total_pages: analysis_result_row.total_pages,
            started_at: analysis_result_row.started_at.map(|dt| dt.and_utc()),
            created_at: analysis_result_row
                .created_at
                .expect("Must Exist")
                .and_utc(),
            completed_at: analysis_result_row.completed_at.map(|dt| dt.and_utc()),
            sitemap_found: analysis_result_row.sitemap_found,
            robots_txt_found: analysis_result_row.robots_txt_found,
            ssl_certificate: analysis_result_row.ssl_certificate,
        };

        let issues_rows = sqlx::query!(
            r#"
            SELECT si.type, si.title, si.description, si.page_url, si.element, si.line_number, si.recommendation
            FROM seo_issues si
            JOIN page_analysis pa ON si.page_id = pa.id
            WHERE pa.analysis_id = ?
            "#,
            analysis_result_row.id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch SEO issues for analysis result")?;

        let issues: Vec<SeoIssue> = issues_rows
            .into_iter()
            .map(|row| SeoIssue {
                page_id: "".to_string(), // page_id is not needed here
                issue_type: map_issue_type(&row.r#type),
                title: row.title,
                description: row.description,
                page_url: row.page_url,
                element: row.element,
                line_number: row.line_number,
                recommendation: row.recommendation,
            })
            .collect();

        let rows = sqlx::query!(
            r#"
        SELECT 
            pa.id,
            pa.analysis_id,
            pa.url,
            pa.title,
            pa.meta_description,
            pa.meta_keywords,
            pa.canonical_url,
            pa.h1_count,
            pa.h2_count,
            pa.h3_count,
            pa.word_count,
            pa.image_count,
            pa.images_without_alt,
            pa.internal_links,
            pa.external_links,
            pa.load_time,
            pa.status_code     AS page_status,
            pa.content_size,
            pa.mobile_friendly,
            pa.has_structured_data,
            pa.lighthouse_performance,
            pa.lighthouse_accessibility,
            pa.lighthouse_best_practices,
            pa.lighthouse_seo,
            pa.created_at,
            pa.headings,
            pa.images,
            pa.links as detailed_links_json,

            GROUP_CONCAT(pe.to_url)        AS edge_urls,
            GROUP_CONCAT(CAST(pe.status_code as TEXT))   AS edge_statuses

        FROM page_analysis pa
        LEFT JOIN page_edge pe ON pe.from_page_id = pa.id
        WHERE pa.analysis_id = ?
        GROUP BY pa.id
        ORDER BY pa.id;
    "#,
            analysis_result_row.id
        )
        .fetch_all(&self.pool)
        .await?;

        let pages: Vec<PageAnalysisData> = rows
            .into_iter()
            .map(|r| {
                let links: Vec<PageEdge> = match (r.edge_urls, r.edge_statuses) {
                    (Some(urls), Some(sts)) => {
                        let url_vec: Vec<_> = urls.split(',').map(str::to_owned).collect();
                        let status_vec: Vec<u16> =
                            sts.split(',').filter_map(|s| s.parse().ok()).collect();

                        url_vec
                            .into_iter()
                            .zip(status_vec)
                            .map(|(u, s)| PageEdge {
                                from_page_id: r.id.clone().unwrap(),
                                to_url: u,
                                status_code: s,
                            })
                            .collect()
                    }
                    _ => Vec::new(),
                };
                PageAnalysisData {
                    analysis_id: r.analysis_id,
                    url: r.url,
                    title: r.title,
                    meta_description: r.meta_description,
                    meta_keywords: r.meta_keywords,
                    canonical_url: r.canonical_url,
                    h1_count: r.h1_count,
                    h2_count: r.h2_count,
                    h3_count: r.h3_count,
                    word_count: r.word_count,
                    image_count: r.image_count,
                    images_without_alt: r.images_without_alt,
                    internal_links: r.internal_links,
                    external_links: r.external_links,
                    load_time: r.load_time,
                    status_code: r.page_status,
                    content_size: r.content_size,
                    mobile_friendly: r.mobile_friendly,
                    has_structured_data: r.has_structured_data,
                    lighthouse_performance: r.lighthouse_performance,
                    lighthouse_accessibility: r.lighthouse_accessibility,
                    lighthouse_best_practices: r.lighthouse_best_practices,
                    lighthouse_seo: r.lighthouse_seo,
                    links,
                    headings: r
                        .headings
                        .and_then(|h| serde_json::from_str::<Vec<HeadingElement>>(&h).ok())
                        .unwrap_or_default(),
                    images: r
                        .images
                        .and_then(|i| serde_json::from_str::<Vec<ImageElement>>(&i).ok())
                        .unwrap_or_default(),
                    detailed_links: r
                        .detailed_links_json
                        .and_then(|l| serde_json::from_str::<Vec<LinkElement>>(&l).ok())
                        .unwrap_or_default(),
                }
            })
            .collect();

        // Populate status codes for internal links if we have crawled them
        let url_status: HashMap<String, Option<i64>> = pages
            .iter()
            .map(|p| (p.url.clone(), p.status_code))
            .collect();

        let mut pages = pages;
        for page in &mut pages {
            if let Ok(base_url) = Url::parse(&page.url) {
                for link in &mut page.detailed_links {
                    if link.status_code.is_none() {
                        // Only check internal links or links we might have visited
                        if let Ok(abs_url) = base_url.join(&link.href) {
                            let abs_str = abs_url.to_string();
                            // Try exact match
                            if let Some(Some(code)) = url_status.get(&abs_str) {
                                link.status_code = Some(*code as u16);
                            } else {
                                // Try trimming internal anchor hash if present
                                let mut abs_no_frag = abs_url.clone();
                                abs_no_frag.set_fragment(None);
                                if let Some(Some(code)) = url_status.get(abs_no_frag.as_str()) {
                                    link.status_code = Some(*code as u16);
                                }
                            }
                        }
                    }
                }
            }
        }

        let summay_row = sqlx::query!(
            r#"
            SELECT seo_score, avg_load_time, total_words, pages_with_issues
            FROM analysis_summary
            WHERE analysis_id = ?
            "#,
            analysis_result_row.id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch analysis summary")?;

        let summary = AnalysisSummary {
            analysis_id: analysis_result_row.id.clone(),
            seo_score: summay_row.seo_score,
            avg_load_time: summay_row.avg_load_time,
            total_words: summay_row.total_words,
            total_issues: summay_row.pages_with_issues,
        };
        let complete_analysis = CompleteAnalysisResult {
            analysis,
            issues,
            pages,
            summary,
        };

        Ok(complete_analysis)
    }
}
