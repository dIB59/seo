use std::sync::Arc;

use anyhow::Result;

use crate::contexts::report::domain::ReportData;
use crate::repository::{ReportPatternRepository, ResultsRepository};

use super::{brief_builder, pattern_engine};

pub struct ReportService {
    pattern_repo: Arc<dyn ReportPatternRepository>,
    results_repo: Arc<dyn ResultsRepository>,
}

impl ReportService {
    pub fn new(
        pattern_repo: Arc<dyn ReportPatternRepository>,
        results_repo: Arc<dyn ResultsRepository>,
    ) -> Self {
        Self { pattern_repo, results_repo }
    }

    /// Generate a full report for the given job.
    pub async fn generate_report(&self, job_id: &str) -> Result<ReportData> {
        let result = self.results_repo.get_complete_result(job_id).await?;
        let patterns = self.pattern_repo.list_enabled_patterns().await?;

        let detected = pattern_engine::evaluate_all(&patterns, &result);
        let pillar_scores = pattern_engine::compute_pillar_scores(&detected);
        let ai_brief = brief_builder::build_ai_brief(&result.job, &detected, &pillar_scores);

        let job = &result.job;
        Ok(ReportData {
            job_id: job.id.clone(),
            url: job.url.clone(),
            seo_score: job.calculate_seo_score(),
            total_pages: job.summary.total_pages,
            total_issues: job.summary.total_issues,
            critical_issues: job.summary.critical_issues,
            warning_issues: job.summary.warning_issues,
            sitemap_found: job.sitemap_found,
            robots_txt_found: job.robots_txt_found,
            pillar_scores,
            detected_patterns: detected,
            ai_brief,
        })
    }
}
