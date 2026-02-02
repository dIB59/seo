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
        let query_start = std::time::Instant::now();

        // Step 1: Get analysis metadata
        let analysis_result_row = sqlx::query!(
            r#"
            SELECT ar.id as "id!", ar.url, ar.status, ar.progress, ar.analyzed_pages, ar.total_pages,
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

        let analysis_id = analysis_result_row.id.clone();

        let analysis = AnalysisResults {
            id: analysis_id.clone(),
            url: analysis_result_row.url.clone(),
            status: map_job_status(&analysis_result_row.status),
            progress: analysis_result_row.progress,
            analyzed_pages: analysis_result_row.analyzed_pages,
            total_pages: analysis_result_row.total_pages,
            started_at: analysis_result_row.started_at.map(|dt| dt.and_utc()),
            created_at: analysis_result_row.created_at.and_utc(),
            completed_at: analysis_result_row.completed_at.map(|dt| dt.and_utc()),
            sitemap_found: analysis_result_row.sitemap_found,
            robots_txt_found: analysis_result_row.robots_txt_found,
            ssl_certificate: analysis_result_row.ssl_certificate,
        };

        // Step 2: Get issues (unchanged)
        let issues_rows = sqlx::query!(
            r#"
            SELECT si.type, si.title, si.description, si.page_url, si.element, si.line_number, si.recommendation
            FROM seo_issues si
            JOIN page_analysis pa ON si.page_id = pa.id
            WHERE pa.analysis_id = ?
            "#,
            analysis_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch SEO issues for analysis result")?;

        let issues: Vec<SeoIssue> = issues_rows
            .into_iter()
            .map(|row| SeoIssue {
                page_id: "".to_string(),
                issue_type: map_issue_type(&row.r#type),
                title: row.title,
                description: row.description,
                page_url: row.page_url,
                element: row.element,
                line_number: row.line_number,
                recommendation: row.recommendation,
            })
            .collect();

        // Step 3: Get ALL pages WITHOUT GROUP_CONCAT (FAST!)
        let pages_start = std::time::Instant::now();
        let page_rows = sqlx::query!(
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
                pa.status_code AS page_status,
                pa.content_size,
                pa.mobile_friendly,
                pa.has_structured_data,
                pa.lighthouse_performance,
                pa.lighthouse_accessibility,
                pa.lighthouse_best_practices,
                pa.lighthouse_seo,
                pa.lighthouse_seo_audits,
                pa.lighthouse_performance_metrics,
                pa.created_at,
                pa.headings,
                pa.images,
                pa.links as detailed_links_json
            FROM page_analysis pa
            WHERE pa.analysis_id = ?
            ORDER BY pa.id
            "#,
            analysis_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pages")?;

        log::debug!(
            "Fetched {} pages in {:?}",
            page_rows.len(),
            pages_start.elapsed()
        );

        // Step 4: Get ALL edges in one separate query (FAST!)
        let edges_start = std::time::Instant::now();
        let edge_rows = sqlx::query!(
            r#"
            SELECT pe.from_page_id, pe.to_url, pe.status_code
            FROM page_edge pe
            WHERE pe.from_page_id IN (
                SELECT id FROM page_analysis WHERE analysis_id = ?
            )
            "#,
            analysis_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch edges")?;

        log::debug!(
            "Fetched {} edges in {:?}",
            edge_rows.len(),
            edges_start.elapsed()
        );

        // Step 5: Group edges by page in Rust (VERY FAST - microseconds!)
        let grouping_start = std::time::Instant::now();
        let mut edges_by_page: HashMap<String, Vec<PageEdge>> = HashMap::new();

        for edge in edge_rows {
            let page_edge = PageEdge {
                from_page_id: edge.from_page_id.clone(),
                to_url: edge.to_url,
                status_code: edge.status_code as u16,
            };

            edges_by_page
                .entry(edge.from_page_id)
                .or_insert_with(Vec::new)
                .push(page_edge);
        }

        log::debug!(
            "Grouped edges into {} page groups in {:?}",
            edges_by_page.len(),
            grouping_start.elapsed()
        );

        // Step 6: Build URL status lookup for internal links
        let url_status: HashMap<String, Option<i64>> = page_rows
            .iter()
            .map(|p| (p.url.clone(), p.page_status))
            .collect();

        // Step 7: Construct pages with edges
        let parsing_start = std::time::Instant::now();
        let mut pages: Vec<PageAnalysisData> = Vec::with_capacity(page_rows.len());

        for row in page_rows {
            // Get edges for this page (fast HashMap lookup)
            let row_id = row.id;
            let links = edges_by_page
                .remove(&row_id.expect("Page Id must exist"))
                .unwrap_or_default();

            // Parse JSON fields
            let headings = row
                .headings
                .and_then(|h| serde_json::from_str::<Vec<HeadingElement>>(&h).ok())
                .unwrap_or_default();

            let images = row
                .images
                .and_then(|i| serde_json::from_str::<Vec<ImageElement>>(&i).ok())
                .unwrap_or_default();

            let mut detailed_links = row
                .detailed_links_json
                .and_then(|l| serde_json::from_str::<Vec<LinkElement>>(&l).ok())
                .unwrap_or_default();

            // Populate status codes for internal links
            if let Ok(base_url) = Url::parse(&row.url) {
                for link in &mut detailed_links {
                    if link.status_code.is_none() {
                        if let Ok(abs_url) = base_url.join(&link.href) {
                            let abs_str = abs_url.to_string();

                            // Try exact match
                            if let Some(Some(code)) = url_status.get(&abs_str) {
                                link.status_code = Some(*code as u16);
                            } else {
                                // Try without fragment
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

            pages.push(PageAnalysisData {
                analysis_id: row.analysis_id,
                url: row.url,
                title: row.title,
                meta_description: row.meta_description,
                meta_keywords: row.meta_keywords,
                canonical_url: row.canonical_url,
                h1_count: row.h1_count,
                h2_count: row.h2_count,
                h3_count: row.h3_count,
                word_count: row.word_count,
                image_count: row.image_count,
                images_without_alt: row.images_without_alt,
                internal_links: row.internal_links,
                external_links: row.external_links,
                load_time: row.load_time,
                status_code: row.page_status,
                content_size: row.content_size,
                mobile_friendly: row.mobile_friendly,
                has_structured_data: row.has_structured_data,
                lighthouse_performance: row.lighthouse_performance,
                lighthouse_accessibility: row.lighthouse_accessibility,
                lighthouse_best_practices: row.lighthouse_best_practices,
                lighthouse_seo: row.lighthouse_seo,
                lighthouse_seo_audits: row.lighthouse_seo_audits
                    .as_ref()
                    .and_then(|s| serde_json::from_str(s).ok()),
                lighthouse_performance_metrics: row.lighthouse_performance_metrics
                    .as_ref()
                    .and_then(|s| serde_json::from_str(s).ok()),
                links,
                headings,
                images,
                detailed_links,
            });
        }

        log::debug!(
            "Parsed {} pages in {:?}",
            pages.len(),
            parsing_start.elapsed()
        );

        // Step 8: Get summary
        let summary_row = sqlx::query!(
            r#"
            SELECT seo_score, avg_load_time, total_words, pages_with_issues
            FROM analysis_summary
            WHERE analysis_id = ?
            "#,
            analysis_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch analysis summary")?;

        let summary = AnalysisSummary {
            analysis_id: analysis_id.clone(),
            seo_score: summary_row.seo_score,
            avg_load_time: summary_row.avg_load_time,
            total_words: summary_row.total_words,
            total_issues: summary_row.pages_with_issues,
        };

        let total_time = query_start.elapsed();
        log::info!(
            "Loaded analysis for job {} with {} pages, {} edges, {} issues in {:?}",
            job_id,
            pages.len(),
            edges_by_page.len(),
            issues.len(),
            total_time
        );

        Ok(CompleteAnalysisResult {
            analysis,
            issues,
            pages,
            summary,
        })
    }
}

mod tests {

    #[tokio::test]
    #[ignore]
    async fn test_get_result_by_job_id() {
        let pool = crate::test_utils::set_up_test_db_with_prod_data().await;
        let repo = crate::repository::sqlite::ResultsRepository::new(pool);

        let result = repo.get_result_by_job_id(12).await.unwrap();
        assert_eq!(result.pages.len(), 1497);
        assert_eq!(result.issues.len(), 2576);
    }
}
