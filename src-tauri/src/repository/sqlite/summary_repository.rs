use sqlx::SqlitePool;

use crate::domain::models::{IssueType, PageAnalysisData, SeoIssue};
use anyhow::Result;

#[derive(Clone)]
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
        let average_load_time = analyzed_page_data.iter().map(|d| d.load_time).sum::<f64>()
            / analyzed_page_data.len() as f64;

        let total_words = analyzed_page_data.iter().map(|d| d.word_count).sum::<i64>();

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
