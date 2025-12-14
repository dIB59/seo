use crate::{
    commands::analysis::AnalysisSettingsRequest, domain::models::*,
    service::job_processor::PageEdge,
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use uuid::Uuid;

// ====== Mappers ======

fn map_job_status(s: &str) -> JobStatus {
    match s {
        "queued" => JobStatus::Queued,
        "processing" => JobStatus::Processing,
        "completed" => JobStatus::Completed,
        "failed" => JobStatus::Failed,
        _ => JobStatus::Queued,
    }
}

fn map_issue_type(s: &str) -> IssueType {
    match s {
        "critical" => IssueType::Critical,
        "warning" => IssueType::Warning,
        "suggestion" => IssueType::Suggestion,
        _ => IssueType::Suggestion,
    }
}

// ====== Repositories ======

pub struct JobRepository {
    pool: SqlitePool,
}

impl JobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_pending_jobs(&self) -> Result<Vec<AnalysisJob>> {
        let rows = sqlx::query!(
            "SELECT id, url, settings_id, created_at, status, result_id \
             FROM analysis_jobs \
             WHERE status IN ('queued', 'processing') \
             ORDER BY created_at ASC
             "
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| AnalysisJob {
                id: row.id.expect("ID must not be null"),
                url: row.url,
                settings_id: row.settings_id,
                created_at: row
                    .created_at
                    .expect("Created at must not be null")
                    .and_utc(),
                status: map_job_status(&row.status),
                result_id: row.result_id,
            })
            .collect())
    }

    pub async fn update_status(&self, job_id: i64, status: JobStatus) -> Result<()> {
        log::info!("Job {} being updated to {:?}", job_id, status);
        sqlx::query("UPDATE analysis_jobs SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(job_id)
            .execute(&self.pool)
            .await
            .context("Failed to update job status")?;
        Ok(())
    }

    pub async fn create_with_settings(
        &self,
        url: &str,
        settings: &AnalysisSettingsRequest,
    ) -> Result<i64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction")?;
        // Insert settings
        let max_pages = settings.max_pages;
        let include_external_links = settings.include_external_links as i64;
        let check_images = settings.check_images as i64;
        let mobile_analysis = settings.mobile_analysis as i64;
        let lighthouse_analysis = settings.lighthouse_analysis as i64;
        let delay_between_requests = settings.delay_between_requests;

        let settings_id = sqlx::query_scalar!(
            r#"
            INSERT INTO analysis_settings (
                max_pages, 
                include_external_links, 
                check_images, 
                mobile_analysis, 
                lighthouse_analysis, 
                delay_between_requests
            )
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
            max_pages,
            include_external_links,
            check_images,
            mobile_analysis,
            lighthouse_analysis,
            delay_between_requests
        )
        .fetch_one(tx.as_mut())
        .await
        .context("Failed to insert settings")?;

        // Insert job
        let job_id = sqlx::query_scalar!(
            r#"
            INSERT INTO analysis_jobs (url, settings_id, status) 
            VALUES (?, ?, 'queued') 
            RETURNING id
            "#,
            url,
            settings_id
        )
        .fetch_one(tx.as_mut())
        .await
        .context("Failed to insert analysis job")?;

        tx.commit().await.context("Failed to commit transaction")?;

        log::info!("Analysis job {} created successfully", job_id);
        Ok(job_id)
    }

    pub async fn get_progress(&self, job_id: i64) -> Result<AnalysisProgress> {
        let row = sqlx::query_as!(
            AnalysisProgress,
            r#"
            SELECT 
                aj.id as job_id,
                aj.url,
                aj.status as job_status,
                aj.result_id,
                ar.progress,
                ar.analyzed_pages,
                ar.total_pages
            FROM analysis_jobs aj
            LEFT JOIN analysis_results ar ON aj.result_id = ar.id
            WHERE aj.id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch analysis progress")?;

        Ok(row)
    }

    pub async fn get_all(&self) -> Result<Vec<AnalysisProgress>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                aj.id as "job_id!",
                aj.url,
                aj.status as job_status,
                aj.result_id,
                ar.status as analysis_status,
                ar.progress,
                ar.analyzed_pages,
                ar.total_pages
            FROM analysis_jobs aj
            LEFT JOIN analysis_results ar ON aj.result_id = ar.id
            ORDER BY aj.created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch analysis jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| AnalysisProgress {
                job_id: row.job_id,
                url: row.url,
                job_status: row.job_status,
                result_id: row.result_id,
                progress: row.progress,
                analyzed_pages: row.analyzed_pages,
                total_pages: row.total_pages,
            })
            .collect())
    }

    pub async fn link_to_result(&self, job_id: i64, result_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE analysis_jobs 
            SET result_id = ? 
            WHERE id = ?
            "#,
            result_id,
            job_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to link job to result")?;

        log::info!("Linked job {} to result {}", job_id, result_id);
        Ok(())
    }
}

pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: i64) -> Result<AnalysisSettings> {
        let row = sqlx::query!("SELECT * FROM analysis_settings WHERE id = ?", id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to fetch analysis settings")?;

        Ok(AnalysisSettings {
            id: row.id,
            max_pages: row.max_pages,
            include_external_links: row.include_external_links != 0,
            check_images: row.check_images != 0,
            mobile_analysis: row.mobile_analysis != 0,
            lighthouse_analysis: row.lighthouse_analysis != 0,
            delay_between_requests: row.delay_between_requests,
            created_at: row.created_at.expect("Must exist").and_utc(),
        })
    }
}

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
                }
            })
            .collect();

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

pub struct PageRepository {
    pool: SqlitePool,
}

impl PageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, page: &PageAnalysisData) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO page_analysis (id, analysis_id, url, title, meta_description, meta_keywords, \
             canonical_url, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, \
             internal_links, external_links, load_time, status_code, content_size, mobile_friendly, \
             has_structured_data, lighthouse_performance, lighthouse_accessibility, \
             lighthouse_best_practices, lighthouse_seo, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&page.analysis_id)
        .bind(&page.url)
        .bind(&page.title)
        .bind(&page.meta_description)
        .bind(&page.meta_keywords)
        .bind(&page.canonical_url)
        .bind(page.h1_count)
        .bind(page.h2_count)
        .bind(page.h3_count)
        .bind(page.word_count)
        .bind(page.image_count)
        .bind(page.images_without_alt)
        .bind(page.internal_links)
        .bind(page.external_links)
        .bind(page.load_time)
        .bind(page.status_code)
        .bind(page.content_size)
        .bind(page.mobile_friendly)
        .bind(page.has_structured_data)
        .bind(page.lighthouse_performance)
        .bind(page.lighthouse_accessibility)
        .bind(page.lighthouse_best_practices)
        .bind(page.lighthouse_seo)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .context("Failed to insert page analysis")?;

        Ok(id)
    }

    pub(crate) async fn insert_edges_batch(&self, edges: &[PageEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }
        let mut qb =
            sqlx::QueryBuilder::new("INSERT INTO page_edge (from_page_id, to_url, status_code) ");
        qb.push_values(edges, |mut b, edge| {
            b.push_bind(&edge.from_page_id)
                .push_bind(&edge.to_url)
                .push_bind(edge.status_code as i32);
        });
        qb.build().execute(&self.pool).await?;
        Ok(())
    }
}

pub struct IssuesRepository {
    pool: SqlitePool,
}

impl IssuesRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_batch(&self, issues: &[SeoIssue]) -> Result<()> {
        if issues.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;
        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO seo_issues (id, page_id, type, title, description, page_url, element, line_number, recommendation) "
        );

        query_builder.push_values(issues, |mut b, issue| {
            b.push_bind(Uuid::new_v4().to_string())
                .push_bind(&issue.page_id)
                .push_bind(issue.issue_type.as_str())
                .push_bind(&issue.title)
                .push_bind(&issue.description)
                .push_bind(&issue.page_url)
                .push_bind(&issue.element)
                .push_bind(issue.line_number)
                .push_bind(&issue.recommendation);
        });

        query_builder.build().execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}

pub struct SummaryRepository {
    pool: SqlitePool,
}

impl SummaryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn generate_summary(
        &self,
        analysis_id: &str,
        issues: &[SeoIssue],
        analyzed_page_data: &[PageAnalysisData],
    ) -> Result<()> {
        let mut critical = 0;
        let mut warnings = 0;
        let mut suggestions = 0;

        for issue in issues {
            match issue.issue_type {
                IssueType::Critical => critical += 1,
                IssueType::Warning => warnings += 1,
                IssueType::Suggestion => suggestions += 1,
            }
        }

        let mut tx = self.pool.begin().await?;
        let average_load_time = analyzed_page_data
            .iter()
            .map(|d| d.load_time)
            .reduce(|a, b| a + b)
            .unwrap_or_default();

        let total_words = analyzed_page_data
            .iter()
            .map(|d| d.word_count)
            .reduce(|a, b| a + b)
            .unwrap_or_default();

        sqlx::query(
            "INSERT OR REPLACE INTO analysis_issues (analysis_id, critical, warnings, suggestions) \
             VALUES (?, ?, ?, ?)"
        )
        .bind(analysis_id)
        .bind(critical)
        .bind(warnings)
        .bind(suggestions)
        .execute(&mut *tx)
        .await?;
        //TODO:
        //FIX HARD CODED VALUES
        sqlx::query(
            "INSERT OR REPLACE INTO analysis_summary \
             (analysis_id, seo_score, avg_load_time, total_words, pages_with_issues) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(analysis_id)
        .bind(75)
        .bind(average_load_time)
        .bind(total_words)
        .bind(if issues.is_empty() {
            0
        } else {
            analyzed_page_data.len() as i64
        })
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Adjust path if necessary, but default often works for crate root
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_job_lifecycle() {
        let pool = setup_db().await;
        let repo = JobRepository::new(pool.clone());

        let settings = AnalysisSettingsRequest {
            max_pages: 5,
            delay_between_requests: 100,
            ..Default::default()
        };

        // 1. Create
        let job_id = repo
            .create_with_settings("https://test.com", &settings)
            .await
            .expect("Failed to create job");

        // 2. Verify Pending
        let pending = repo.get_pending_jobs().await.expect("Failed to get pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, job_id);
        assert_eq!(pending[0].status, JobStatus::Queued);

        // 3. Update Status
        repo.update_status(job_id, JobStatus::Processing)
            .await
            .expect("Update status failed");
        
        let pending_processing = repo.get_pending_jobs().await.unwrap();
        assert_eq!(pending_processing[0].status, JobStatus::Processing);

        // 4. Complete
        repo.update_status(job_id, JobStatus::Completed)
            .await
            .expect("Update status failed");
        
        let pending_final = repo.get_pending_jobs().await.unwrap();
        assert!(pending_final.is_empty());
    }

    #[tokio::test]
    async fn test_settings_persistence() {
        let pool = setup_db().await;
        let job_repo = JobRepository::new(pool.clone());
        let settings_repo = SettingsRepository::new(pool.clone());

        let settings = AnalysisSettingsRequest {
            max_pages: 42,
            ..Default::default()
        };

        // Creating a job also creates settings
        let job_id = job_repo.create_with_settings("https://settings.test", &settings).await.unwrap();
        
        let pending = job_repo.get_pending_jobs().await.unwrap();
        let settings_id = pending[0].settings_id;

        let retrieved = settings_repo.get_by_id(settings_id).await.unwrap();
        assert_eq!(retrieved.max_pages, 42);
    }

    #[tokio::test]
    async fn test_results_and_pages() {
        let pool = setup_db().await;
        let results_repo = ResultsRepository::new(pool.clone());
        let page_repo = PageRepository::new(pool.clone());
        let job_repo = JobRepository::new(pool.clone());
        
        let settings = AnalysisSettingsRequest::default();
        let job_id = job_repo.create_with_settings("https://result.test", &settings).await.unwrap();

        // 1. Create Result
        let result_id = results_repo.create(
            "https://result.test",
            true, // sitemap
            true, // robots
            true  // ssl
        ).await.expect("Failed to create result");

        // Link
        job_repo.link_to_result(job_id, &result_id).await.unwrap();

        // 2. Add a page
        let page_data = PageAnalysisData {
            analysis_id: result_id.clone(),
            url: "https://result.test/page1".into(),
            title: Some("Page 1".into()),
            meta_description: None,
            meta_keywords: None,
            canonical_url: None,
            h1_count: 1,
            h2_count: 0,
            h3_count: 0,
            word_count: 100,
            image_count: 1,
            images_without_alt: 0,
            internal_links: 0,
            external_links: 0,
            load_time: 0.2,
            status_code: Some(200),
            content_size: 500,
            mobile_friendly: true,
            has_structured_data: false,
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            links: vec![],
        };

        page_repo.insert(&page_data).await.expect("Failed to insert page");

        // 3. Update Progress
        results_repo.update_progress(&result_id, 50.0, 1, 2).await.unwrap();
        
        // Generate summary (required for get_result_by_job_id)
        let summary_repo = SummaryRepository::new(pool.clone());
        summary_repo.generate_summary(&result_id, &[], &[page_data]).await.unwrap();

        let complete = results_repo.get_result_by_job_id(job_id).await.unwrap();
        assert_eq!(complete.analysis.id, result_id);
        assert_eq!(complete.analysis.progress, 50.0);
        assert_eq!(complete.pages.len(), 1);
        assert_eq!(complete.pages[0].url, "https://result.test/page1");

        // 4. Finalize
        results_repo.finalize(&result_id, AnalysisStatus::Completed).await.unwrap();

        let finalized = results_repo.get_result_by_job_id(job_id).await.unwrap();
        assert_eq!(finalized.analysis.status, JobStatus::Completed); // check mapper logic if it matches enum
    }
}
