use tauri_specta::{collect_commands, Commands};

pub(super) trait ResultExt<T> {
    fn context(self, msg: &str) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> Result<T, String> {
        self.map_err(|e| format!("{}: {}", msg, e))
    }
}

mod ai;
pub mod analysis;
mod extension;
mod licensing;
mod local_model;
mod report;

pub fn register_commands() -> Commands<tauri::Wry> {
    collect_commands![
        // AI commands
        ai::get_gemini_insights,
        ai::get_gemini_api_key,
        ai::set_gemini_api_key,
        ai::get_gemini_persona,
        ai::set_gemini_persona,
        ai::get_gemini_requirements,
        ai::set_gemini_requirements,
        ai::get_gemini_context_options,
        ai::set_gemini_context_options,
        ai::get_gemini_prompt_blocks,
        ai::set_gemini_prompt_blocks,
        ai::get_gemini_enabled,
        ai::set_gemini_enabled,
        ai::get_ai_source,
        ai::set_ai_source,
        // Analysis commands
        analysis::start_analysis,
        analysis::get_analysis_progress,
        analysis::get_all_jobs,
        analysis::get_paginated_jobs,
        analysis::cancel_analysis,
        analysis::get_result,
        analysis::get_analysis_defaults,
        analysis::get_free_tier_defaults,
        // Licensing commands
        licensing::activate_license,
        licensing::activate_with_key,
        licensing::get_user_policy,
        licensing::get_license_tier,
        licensing::get_machine_id,
        // Local model commands
        local_model::list_local_models,
        local_model::download_local_model,
        local_model::cancel_model_download,
        local_model::delete_local_model,
        local_model::get_active_local_model,
        local_model::set_active_local_model,
        local_model::generate_local_insights,
        // Extension commands
        extension::list_custom_checks,
        extension::create_custom_check,
        extension::update_custom_check,
        extension::delete_custom_check,
        extension::list_custom_extractors,
        extension::create_custom_extractor,
        extension::update_custom_extractor,
        extension::delete_custom_extractor,
        extension::list_tags,
        // Report commands
        report::list_report_patterns,
        report::create_report_pattern,
        report::update_report_pattern,
        report::toggle_report_pattern,
        report::delete_report_pattern,
        report::generate_report_data,
        // Report template commands
        report::list_report_templates,
        report::get_report_template,
        report::create_report_template,
        report::update_report_template,
        report::set_active_report_template,
        report::delete_report_template,
    ]
}

#[cfg(test)]
mod tests {
    //! Characterization tests for the `ResultExt::context` helper used
    //! by every command boundary to convert internal errors to the
    //! frontend-facing `String` payload. The format is the wire format
    //! Tauri sends back, so any change here is observable.

    use super::ResultExt;

    #[test]
    fn context_prefixes_message_with_separator() {
        let r: Result<i64, std::io::Error> = Err(std::io::Error::other("boom"));
        let prefixed = r.context("step failed");
        assert_eq!(prefixed.unwrap_err(), "step failed: boom");
    }

    #[test]
    fn context_passes_through_ok_unchanged() {
        let r: Result<i64, std::io::Error> = Ok(42);
        assert_eq!(r.context("never used").unwrap(), 42);
    }

    #[test]
    fn context_uses_display_not_debug() {
        // The Display impl of std::io::Error doesn't include the kind
        // discriminant, so the formatted message must not include
        // "Custom { kind:" — pinning that we use Display, not Debug.
        let r: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
        let err = r.context("ctx").unwrap_err();
        assert_eq!(err, "ctx: missing");
        assert!(!err.contains("kind:"));
        assert!(!err.contains("Custom"));
    }

    #[test]
    fn context_works_with_anyhow() {
        let r: Result<(), anyhow::Error> = Err(anyhow::anyhow!("root cause"));
        let err = r.context("operation").unwrap_err();
        assert_eq!(err, "operation: root cause");
    }

    #[test]
    fn context_does_not_unwind_anyhow_chain() {
        // Display on anyhow::Error only shows the outermost message,
        // not the source chain. Pinning that the wire format is the
        // top-level error only — the chain is preserved in
        // CommandError but not in this raw helper.
        let inner = anyhow::anyhow!("inner");
        let outer = inner.context("outer");
        let r: Result<(), anyhow::Error> = Err(outer);
        let err = r.context("ctx").unwrap_err();
        assert_eq!(err, "ctx: outer");
        assert!(!err.contains("inner"));
    }
}

