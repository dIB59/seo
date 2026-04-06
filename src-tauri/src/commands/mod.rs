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
        // Report commands
        report::list_report_patterns,
        report::create_report_pattern,
        report::update_report_pattern,
        report::toggle_report_pattern,
        report::delete_report_pattern,
        report::generate_report_data,
    ]
}

