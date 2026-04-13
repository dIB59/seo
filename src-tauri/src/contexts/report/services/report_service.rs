use std::sync::Arc;

use anyhow::Result;

use crate::contexts::local_model::LocalModelService;
use crate::contexts::report::domain::ReportData;
use crate::contexts::report::template::{
    render_template, RenderContext, RenderedFragment,
};
use crate::repository::{
    ReportPatternRepository, ReportTemplateRepository, ResultsRepository, SettingsRepository,
};
use crate::service::local_model::InferenceRequest;
use crate::service::prompt::load_persona;

use super::{brief_builder, pattern_engine};

pub struct ReportService {
    pattern_repo:  Arc<dyn ReportPatternRepository>,
    results_repo:  Arc<dyn ResultsRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
    template_repo: Arc<dyn ReportTemplateRepository>,
    /// Local model for AI-generated narrative.  `None` when no model is active.
    local_model:   Option<Arc<LocalModelService>>,
}

impl ReportService {
    pub fn new(
        pattern_repo:  Arc<dyn ReportPatternRepository>,
        results_repo:  Arc<dyn ResultsRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
        template_repo: Arc<dyn ReportTemplateRepository>,
    ) -> Self {
        Self { pattern_repo, results_repo, settings_repo, template_repo, local_model: None }
    }

    pub fn with_local_model(
        pattern_repo:  Arc<dyn ReportPatternRepository>,
        results_repo:  Arc<dyn ResultsRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
        template_repo: Arc<dyn ReportTemplateRepository>,
        local_model:   Arc<LocalModelService>,
    ) -> Self {
        Self { pattern_repo, results_repo, settings_repo, template_repo, local_model: Some(local_model) }
    }

    /// Generate a full report for the given job.
    pub async fn generate_report(&self, job_id: &str) -> Result<ReportData> {
        // Both repos now return RepositoryResult; wrap each in an async fut
        // that converts to anyhow::Error so try_join! sees a single error
        // type.
        let result_fut = async {
            self.results_repo
                .get_complete_result(job_id)
                .await
                .map_err(anyhow::Error::from)
        };
        let patterns_fut = async {
            self.pattern_repo
                .list_enabled_patterns()
                .await
                .map_err(anyhow::Error::from)
        };
        let (result, patterns) = tokio::try_join!(result_fut, patterns_fut)?;

        let detected      = pattern_engine::evaluate_all(&patterns, &result);
        let pillar_scores = pattern_engine::compute_pillar_scores(&detected);

        // Use average lighthouse SEO score — identical to what the app UI displays.
        // Falls back to pillar_scores.overall when lighthouse data is absent.
        let seo_score = {
            let scores: Vec<f64> = result.lighthouse.iter()
                .filter_map(|lh| lh.seo_score)
                .collect();
            if scores.is_empty() {
                pillar_scores.overall().round() as i64
            } else {
                (scores.iter().sum::<f64>() / scores.len() as f64).round() as i64
            }
        };

        let ai_brief = self
            .generate_brief(&result, &detected, &pillar_scores, seo_score)
            .await;

        let job = &result.job;
        Ok(ReportData {
            job_id:          job.id.as_str().to_string(),
            url:             job.url.clone(),
            seo_score,
            total_pages:     job.summary.total_pages(),
            total_issues:    job.summary.total_issues(),
            critical_issues: job.summary.critical_issues(),
            warning_issues:  job.summary.warning_issues(),
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
        result:    &crate::contexts::analysis::CompleteJobResult,
        detected:  &[crate::contexts::report::domain::DetectedPattern],
        pillars:   &crate::contexts::report::domain::PillarScores,
        seo_score: i64,
    ) -> String {
        // Try the template engine first. If an active template exists,
        // render it — expanding AI sections via the local model. This
        // is the unified path that replaces the hardcoded phase1/2/3.
        if let Ok(Some(template)) = self.template_repo.get_active_template().await {
            match self.render_from_template(&template, result, detected, pillars, seo_score).await {
                Ok(brief) => return brief,
                Err(e) => {
                    tracing::warn!(
                        "[Report] Template render failed, falling back to legacy: {e}"
                    );
                }
            }
        }

        // Legacy fallback: hardcoded brief_builder phases.
        self.generate_brief_legacy(&result.job, detected, pillars, seo_score).await
    }

    /// Render a report from a user-authored template. Text, Heading,
    /// PatternSummary, Conditional, and Divider sections are resolved
    /// synchronously. AiPrompt sections are expanded through the local
    /// model (or left as placeholders when no model is active).
    async fn render_from_template(
        &self,
        template:  &crate::contexts::report::ReportTemplate,
        result:    &crate::contexts::analysis::CompleteJobResult,
        detected:  &[crate::contexts::report::domain::DetectedPattern],
        pillars:   &crate::contexts::report::domain::PillarScores,
        seo_score: i64,
    ) -> Result<String> {
        let job = &result.job;

        // Build rich context from the full result so AI prompts get
        // real data — not just bare counts.
        let mut issue_counts_by_page: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        for issue in &result.issues {
            if let Some(pid) = &issue.page_id {
                *issue_counts_by_page.entry(pid.clone()).or_default() += 1;
            }
        }

        // Sort issues by severity for the details block
        let severity_order = |s: &crate::contexts::analysis::IssueSeverity| match s {
            crate::contexts::analysis::IssueSeverity::Critical => 0,
            crate::contexts::analysis::IssueSeverity::Warning => 1,
            crate::contexts::analysis::IssueSeverity::Info => 2,
        };
        let mut sorted_issues = result.issues.clone();
        sorted_issues.sort_by_key(|i| severity_order(&i.severity));

        let issue_details: Vec<String> = sorted_issues
            .iter()
            .take(15)
            .map(|i| {
                let page_url = result
                    .pages
                    .iter()
                    .find(|p| Some(&p.id) == i.page_id.as_ref())
                    .map(|p| p.url.as_str())
                    .unwrap_or("(site-level)");
                format!(
                    "{} | {} | {} | {}",
                    i.severity.as_str(),
                    i.issue_type,
                    page_url,
                    i.message
                )
            })
            .collect();

        // Pages sorted by issue count descending
        let mut pages_sorted: Vec<_> = result.pages.iter().collect();
        pages_sorted.sort_by(|a, b| {
            let ca = issue_counts_by_page.get(&a.id).unwrap_or(&0);
            let cb = issue_counts_by_page.get(&b.id).unwrap_or(&0);
            cb.cmp(ca)
        });

        let page_summaries: Vec<String> = pages_sorted
            .iter()
            .take(10)
            .map(|p| {
                let issues = issue_counts_by_page.get(&p.id).unwrap_or(&0);
                let title = p.title.as_deref().unwrap_or("(no title)");
                let status = p.status_code.map(|s| s.to_string()).unwrap_or_else(|| "?".into());
                let load = p.load_time_ms.map(|t| format!("{t}ms")).unwrap_or_else(|| "?".into());
                format!("{} | {} | {} | {} | {} issues", p.url, title, status, load, issues)
            })
            .collect();

        let missing_meta_count = result
            .pages
            .iter()
            .filter(|p| p.meta_description.is_none())
            .count() as i64;
        let slow_pages_count = result
            .pages
            .iter()
            .filter(|p| p.load_time_ms.is_some_and(|t| t > 3000))
            .count() as i64;
        let error_pages_count = result
            .pages
            .iter()
            .filter(|p| p.status_code.is_some_and(|s| s >= 400))
            .count() as i64;

        let total_load: f64 = result.pages.iter().filter_map(|p| p.load_time_ms).map(|t| t as f64).sum();
        let load_count = result.pages.iter().filter(|p| p.load_time_ms.is_some()).count().max(1);
        let avg_load_time_ms = total_load / load_count as f64;

        let total_words: i64 = result.pages.iter().filter_map(|p| p.word_count).sum();

        // Aggregate custom extractor tag values across all pages,
        // filtered to only the tags the template has selected. Empty
        // selected_tags = include all (backwards compat).
        let mut tag_values = crate::service::prompt::aggregate_tag_values(&result.pages);
        if !template.selected_tags.is_empty() {
            tag_values.retain(|k, _| template.selected_tags.contains(k));
        }

        let ctx = RenderContext {
            job,
            detected,
            pillars,
            seo_score,
            issue_details,
            page_summaries,
            missing_meta_count,
            slow_pages_count,
            error_pages_count,
            avg_load_time_ms,
            total_words,
            tag_values,
        };

        let fragments = render_template(template, &ctx)
            .map_err(|e| anyhow::anyhow!("template render: {e}"))?;

        // Resolve the model path once — shared across all AI sections.
        let model_path = self.resolve_model_path().await;

        // Load persona once — prepended to every AI prompt.
        let persona = load_persona(self.settings_repo.as_ref())
            .await
            .unwrap_or_else(|_| crate::service::prompt::DEFAULT_PERSONA.to_string());

        let mut out = String::new();
        let total = fragments.len();
        for (i, fragment) in fragments.iter().enumerate() {
            match fragment {
                RenderedFragment::Text(s) => out.push_str(s),
                RenderedFragment::AiPrompt { label, prompt } => {
                    tracing::info!(
                        "[Report] Expanding AI section {}/{}: {}",
                        i + 1,
                        total,
                        label
                    );
                    let full_prompt = format!("{persona}\n\n{prompt}");
                    match &model_path {
                        Some(path) => {
                            match self.infer(path, full_prompt, 280).await {
                                Ok(text) if !text.trim().is_empty() => {
                                    out.push_str(text.trim());
                                    out.push_str("\n\n");
                                }
                                Ok(_) => {
                                    tracing::debug!(
                                        "[Report] AI section '{label}' returned empty"
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "[Report] AI section '{label}' failed: {e}"
                                    );
                                }
                            }
                        }
                        None => {
                            // No local model — skip AI sections silently.
                            // The static template sections still render.
                        }
                    }
                }
            }
        }

        Ok(out)
    }

    /// Resolve the active local model's file path. Returns `None` if
    /// no model is configured/downloaded — callers should skip AI
    /// sections in that case.
    async fn resolve_model_path(&self) -> Option<std::path::PathBuf> {
        let lm = self.local_model.as_ref()?;
        let model_id = lm.get_active_model_id().await.ok()??;
        let entry = crate::contexts::local_model::domain::ModelEntry::find_by_id(&model_id)?;
        let path = lm.models_dir().join(&entry.filename);
        if path.exists() { Some(path) } else { None }
    }

    /// Legacy brief generation — the hardcoded phase1/2/3 approach.
    /// Kept as a fallback for when no active template exists or when
    /// template rendering fails.
    async fn generate_brief_legacy(
        &self,
        job:       &crate::contexts::analysis::Job,
        detected:  &[crate::contexts::report::domain::DetectedPattern],
        pillars:   &crate::contexts::report::domain::PillarScores,
        seo_score: i64,
    ) -> String {
        let system_prompt = load_persona(self.settings_repo.as_ref())
            .await
            .unwrap_or_else(|_| crate::service::prompt::DEFAULT_PERSONA.to_string());

        let Some(lm) = &self.local_model else {
            return brief_builder::build_static_brief(job, detected, pillars);
        };

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

        tracing::info!("[Report] Legacy phased AI brief with model {}", model_id);

        let grade   = brief_builder::score_grade(seo_score);
        let weakest = brief_builder::weakest_pillar(pillars);

        let priority_patterns: Vec<_> = detected
            .iter()
            .filter(|d| {
                use crate::contexts::report::domain::PatternSeverity::*;
                d.pattern.severity == Critical || d.pattern.severity == Warning
            })
            .take(3)
            .collect();
        let top_issue_names: Vec<String> = priority_patterns
            .iter()
            .map(|dp| dp.pattern.name.clone())
            .collect();

        let p1 = brief_builder::phase1_diagnosis_prompt(
            &system_prompt,
            &job.url,
            seo_score,
            grade,
            job.summary.total_pages(),
            job.summary.critical_issues(),
            job.summary.warning_issues(),
            job.sitemap_found,
            job.robots_txt_found,
            pillars,
            &top_issue_names,
        );
        let diagnosis = self.infer(&model_path, p1, 280).await.unwrap_or_else(|e| {
            tracing::warn!("[Report] Phase 1 failed: {e}");
            String::new()
        });

        let mut priority_sections = String::new();
        for dp in &priority_patterns {
            let pct = (dp.prevalence * 100.0).round() as u64;
            let business_impact = format!("{:?}", dp.pattern.business_impact);
            let fix_effort = format!("{:?}", dp.pattern.fix_effort);
            let p2 = brief_builder::phase2_issue_prompt(brief_builder::Phase2IssueArgs {
                system_prompt: &system_prompt,
                name: &dp.pattern.name,
                description: &dp.pattern.description,
                pct,
                affected_pages: dp.affected_pages,
                total_pages: dp.total_pages,
                business_impact: &business_impact,
                fix_effort: &fix_effort,
                recommendation: &dp.pattern.recommendation,
            });
            match self.infer(&model_path, p2, 200).await {
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

        let p3 = brief_builder::phase3_roadmap_prompt(&system_prompt, pillars, weakest, &top_issue_names);
        let roadmap = self.infer(&model_path, p3, 320).await.unwrap_or_else(|e| {
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
            job.summary.critical_issues(),
            job.summary.warning_issues(),
            job.summary.total_pages(),
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
        pillars.technical(), pillars.content(), pillars.performance(), pillars.accessibility(),
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
