//! SQLite repository implementations - no extra interfaces

use crate::{analysis::{self, AnalysisSettingsRequest}, domain::models::*};
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

fn map_analysis_status(s: &str) -> AnalysisStatus {
    match s {
        "analyzing" => AnalysisStatus::Analyzing,
        "completed" => AnalysisStatus::Completed,
        "error" => AnalysisStatus::Error,
        "paused" => AnalysisStatus::Paused,
        _ => AnalysisStatus::Analyzing,
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
             FROM analysis_jobs WHERE status = 'queued' ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending jobs")?;

        Ok(rows.into_iter().map(|row| AnalysisJob {
            id: row.id.expect("ID must not be null"),
            url: row.url,
            settings_id: row.settings_id,
            created_at: row.created_at.expect("Created at must not be null").and_utc(),
            status: map_job_status(&row.status),
            result_id: row.result_id,
        }).collect())
    }

    pub async fn update_status(
        &self,
        job_id: i64,
        status: JobStatus
    ) -> Result<()> {
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
        let mut tx = self.pool.begin().await.context("Failed to start transaction")?;
        // Insert settings
        let max_pages = settings.max_pages as i64;
        let include_external_links = settings.include_external_links as i64;
        let check_images = settings.check_images as i64;
        let mobile_analysis = settings.mobile_analysis as i64;
        let lighthouse_analysis = settings.lighthouse_analysis as i64;
        let delay_between_requests = settings.delay_between_requests as i64;

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
                ar.status as analysis_status,
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

        Ok(row.into())
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

        Ok(rows.into_iter().map(|row| AnalysisProgress {
            job_id: row.job_id,
            url: row.url,
            job_status: row.job_status,
            result_id: row.result_id,
            analysis_status: row.analysis_status,
            progress: row.progress,
            analyzed_pages: row.analyzed_pages,
            total_pages: row.total_pages,
        }).collect())
    }

    pub async fn link_to_result(&self, job_id: i64, result_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE analysis_jobs 
            SET result_id = ?, status = 'completed' 
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

    pub async fn create(&self, url: &str, sitemap: bool, robots: bool, ssl: bool) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        sqlx::query(
            "INSERT INTO analysis_results \
             (id, url, status, progress, analyzed_pages, total_pages, started_at, \
              sitemap_found, robots_txt_found, ssl_certificate) \
             VALUES (?, ?, 'analyzing', 0, 0, 0, ?, ?, ?, ?)"
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
        analyzed: i32,
        total: i32,
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
            status: map_analysis_status(&analysis_result_row.status),
            progress: analysis_result_row.progress,
            analyzed_pages: analysis_result_row.analyzed_pages,
            total_pages: analysis_result_row.total_pages,
            started_at: analysis_result_row.started_at.map(|dt| dt.and_utc()),
            created_at: analysis_result_row.created_at.expect("Must Exist").and_utc(),
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

        let issues: Vec<SeoIssue> = issues_rows.into_iter().map(|row| SeoIssue {
            page_id: "".to_string(), // page_id is not needed here
            issue_type: map_issue_type(&row.r#type),
            title: row.title,
            description: row.description,
            page_url: row.page_url,
            element: row.element,
            line_number: row.line_number,
            recommendation: row.recommendation,
        }).collect();

        let pages_rows = sqlx::query!(
            r#"
            SELECT id, analysis_id, url, title, meta_description, meta_keywords,
                   canonical_url, h1_count, h2_count, h3_count, word_count,
                   image_count, images_without_alt, internal_links, external_links,
                   load_time, status_code, content_size, mobile_friendly,
                   has_structured_data, lighthouse_performance,
                   lighthouse_accessibility, lighthouse_best_practices,
                   lighthouse_seo, created_at
            FROM page_analysis
            WHERE analysis_id = ?
            "#,
            analysis_result_row.id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch page analyses for analysis result")?;

        let pages: Vec<PageAnalysisData> = pages_rows.into_iter().map(|row| PageAnalysisData {
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
            status_code: row.status_code,
            content_size: row.content_size,
            mobile_friendly: row.mobile_friendly,
            has_structured_data: row.has_structured_data,
            lighthouse_performance: row.lighthouse_performance,
            lighthouse_accessibility: row.lighthouse_accessibility,
            lighthouse_best_practices: row.lighthouse_best_practices,
            lighthouse_seo: row.lighthouse_seo,
        }).collect();

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
            // critical_issues: todo!(),
            // warning_issues: todo!(),
            // suggestion_issues: todo!(),
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
                .push_bind(&issue.line_number)
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

    pub async fn update_from_issues(
        &self,
        analysis_id: &str,
        issues: &[SeoIssue],
        total_pages: i32,
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

        sqlx::query(
            "INSERT OR REPLACE INTO analysis_summary \
             (analysis_id, seo_score, avg_load_time, total_words, pages_with_issues) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(analysis_id)
        .bind(75)
        .bind(1.5)
        .bind(500)
        .bind(if issues.is_empty() { 0 } else { total_pages })
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}