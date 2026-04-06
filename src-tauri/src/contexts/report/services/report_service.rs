use std::sync::Arc;

use anyhow::Result;

use crate::contexts::local_model::LocalModelService;
use crate::contexts::report::domain::ReportData;
use crate::repository::{ReportPatternRepository, ResultsRepository, SettingsRepository};
use crate::service::local_model::InferenceRequest;
use crate::service::prompt::DEFAULT_PERSONA;

use super::{brief_builder, pattern_engine};

pub struct ReportService {
    pattern_repo:  Arc<dyn ReportPatternRepository>,
    results_repo:  Arc<dyn ResultsRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
    /// Local model for AI-generated narrative.  `None` when no model is active.
    local_model:   Option<Arc<LocalModelService>>,
}

impl ReportService {
    pub fn new(
        pattern_repo:  Arc<dyn ReportPatternRepository>,
        results_repo:  Arc<dyn ResultsRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
    ) -> Self {
        Self { pattern_repo, results_repo, settings_repo, local_model: None }
    }

    pub fn with_local_model(
        pattern_repo:  Arc<dyn ReportPatternRepository>,
        results_repo:  Arc<dyn ResultsRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
        local_model:   Arc<LocalModelService>,
    ) -> Self {
        Self { pattern_repo, results_repo, settings_repo, local_model: Some(local_model) }
    }

    /// Generate a full report for the given job.
    pub async fn generate_report(&self, job_id: &str) -> Result<ReportData> {
        let result   = self.results_repo.get_complete_result(job_id).await?;
        let patterns = self.pattern_repo.list_enabled_patterns().await?;

        let detected      = pattern_engine::evaluate_all(&patterns, &result);
        let pillar_scores = pattern_engine::compute_pillar_scores(&detected);

        // Use average lighthouse SEO score — identical to what the app UI displays.
        // Falls back to pillar_scores.overall when lighthouse data is absent.
        let seo_score = {
            let scores: Vec<f64> = result.lighthouse.iter()
                .filter_map(|lh| lh.seo_score)
                .collect();
            if scores.is_empty() {
                pillar_scores.overall.round() as i64
            } else {
                (scores.iter().sum::<f64>() / scores.len() as f64).round() as i64
            }
        };

        let ai_brief = self
            .generate_brief(&result.job, &detected, &pillar_scores, seo_score)
            .await;

        let job = &result.job;
        Ok(ReportData {
            job_id:          job.id.clone(),
            url:             job.url.clone(),
            seo_score,
            total_pages:     job.summary.total_pages,
            total_issues:    job.summary.total_issues,
            critical_issues: job.summary.critical_issues,
            warning_issues:  job.summary.warning_issues,
            sitemap_found:   job.sitemap_found,
            robots_txt_found: job.robots_txt_found,
            pillar_scores,
            detected_patterns: detected,
            ai_brief,
        })
    }

    // ── Brief generation ──────────────────────────────────────────────────────

    async fn generate_brief(
        &self,
        job:     &crate::contexts::analysis::Job,
        detected: &[crate::contexts::report::domain::DetectedPattern],
        pillars:  &crate::contexts::report::domain::PillarScores,
        seo_score: i64,
    ) -> String {
        // Load the user-configured system prompt — same setting used by Gemini
        // and the local model for regular analysis (single source of truth).
        let system_prompt = match self.settings_repo.get_setting("gemini_persona").await {
            Ok(Some(p)) if !p.trim().is_empty() => p,
            _ => DEFAULT_PERSONA.to_string(),
        };

        // Check whether a local model is active and downloaded.
        let can_use_ai = if let Some(lm) = &self.local_model {
            lm.get_active_model_id().await.ok().flatten().is_some()
        } else {
            false
        };

        if !can_use_ai {
            return brief_builder::build_static_brief(job, detected, pillars);
        }

        let lm = self.local_model.as_ref().unwrap();

        let model_id = match lm.get_active_model_id().await {
            Ok(Some(id)) => id,
            _ => return brief_builder::build_static_brief(job, detected, pillars),
        };

        let model_entry = match crate::contexts::local_model::domain::ModelEntry::find_by_id(&model_id) {
            Some(e) => e,
            None    => return brief_builder::build_static_brief(job, detected, pillars),
        };

        let model_path = lm.models_dir().join(&model_entry.filename);
        if !model_path.exists() {
            return brief_builder::build_static_brief(job, detected, pillars);
        }

        tracing::info!("[Report] Phased AI brief with model {} (persona: {} chars)", model_id, system_prompt.len());

        let grade   = brief_builder::score_grade(seo_score);
        let weakest = brief_builder::weakest_pillar(pillars);

        // ── Phase 1: Diagnosis ────────────────────────────────────────────────
        let p1 = brief_builder::phase1_diagnosis_prompt(
            &system_prompt,
            &job.url,
            seo_score,
            grade,
            job.summary.total_pages,
            job.summary.critical_issues,
            job.summary.warning_issues,
            job.sitemap_found,
            job.robots_txt_found,
        );
        let diagnosis = self.infer(&model_path, p1, 200).await.unwrap_or_else(|e| {
            tracing::warn!("[Report] Phase 1 failed: {e}");
            String::new()
        });

        // ── Phase 2: Priority actions (up to 3 critical/warning patterns) ────
        let priority_patterns: Vec<_> = detected
            .iter()
            .filter(|d| {
                use crate::contexts::report::domain::PatternSeverity::*;
                d.pattern.severity == Critical || d.pattern.severity == Warning
            })
            .take(3)
            .collect();

        let mut priority_sections = String::new();
        for dp in &priority_patterns {
            let pct = (dp.prevalence * 100.0).round() as u64;
            let p2  = brief_builder::phase2_issue_prompt(
                &system_prompt,
                &dp.pattern.name,
                &dp.pattern.description,
                pct,
                &format!("{:?}", dp.pattern.business_impact),
                &format!("{:?}", dp.pattern.fix_effort),
                &dp.pattern.recommendation,
            );
            match self.infer(&model_path, p2, 150).await {
                Ok(text) if !text.trim().is_empty() => {
                    priority_sections.push_str(&format!(
                        "**{}** ({}% of pages)\n{}\n\n",
                        dp.pattern.name, pct, text.trim()
                    ));
                }
                Err(e) => tracing::warn!("[Report] Phase 2 failed for {}: {e}", dp.pattern.name),
                _ => {}
            }
        }

        // ── Phase 3: Roadmap / CTA ────────────────────────────────────────────
        let p3 = brief_builder::phase3_roadmap_prompt(&system_prompt, pillars, weakest);
        let roadmap = self.infer(&model_path, p3, 200).await.unwrap_or_else(|e| {
            tracing::warn!("[Report] Phase 3 failed: {e}");
            String::new()
        });

        assemble_brief(&diagnosis, &priority_sections, &roadmap, job, pillars)
    }

    async fn infer(
        &self,
        model_path: &std::path::Path,
        prompt:     String,
        max_tokens: usize,
    ) -> Result<String> {
        let lm = self.local_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No local model service"))?;

        lm.infer_raw(InferenceRequest {
            model_path:  model_path.to_path_buf(),
            prompt,
            max_tokens,
            temperature: 0.4,
        })
        .await
    }
}

// ── Brief assembly ────────────────────────────────────────────────────────────

fn assemble_brief(
    diagnosis:         &str,
    priority_sections: &str,
    roadmap:           &str,
    job:               &crate::contexts::analysis::Job,
    pillars:           &crate::contexts::report::domain::PillarScores,
) -> String {
    let mut out = String::new();

    out.push_str("## Diagnosis\n\n");
    if !diagnosis.trim().is_empty() {
        out.push_str(diagnosis.trim());
    } else {
        out.push_str(&format!(
            "This site has {} critical issue(s) and {} warning(s) across {} page(s). \
            Immediate action is required to restore search visibility.",
            job.summary.critical_issues,
            job.summary.warning_issues,
            job.summary.total_pages,
        ));
    }
    out.push_str("\n\n");

    if !priority_sections.trim().is_empty() {
        out.push_str("## Priority Actions\n\n");
        out.push_str(priority_sections.trim());
        out.push_str("\n\n");
    }

    out.push_str("## Pillar Health\n\n");
    out.push_str(&format!(
        "- Technical: {:.0}/100\n- Content: {:.0}/100\n\
        - Performance: {:.0}/100\n- Accessibility: {:.0}/100\n\n",
        pillars.technical, pillars.content, pillars.performance, pillars.accessibility,
    ));

    out.push_str("## Next Steps\n\n");
    if !roadmap.trim().is_empty() {
        out.push_str(roadmap.trim());
    } else {
        out.push_str(
            "Address critical issues first, then warnings. \
            Re-audit after each fix sprint.",
        );
    }

    if !job.sitemap_found {
        out.push_str(
            "\n\n**Action required:** Generate and submit a sitemap \
            to Google Search Console.",
        );
    }

    out
}
