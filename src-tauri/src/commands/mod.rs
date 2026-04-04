use tauri_specta::{collect_commands, Commands};

mod ai;
pub mod analysis;
mod extension;
mod licensing;

pub fn register_commands() -> Commands<tauri::Wry> {
    let s = collect_commands![
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
        // Extension commands
        extension::list_custom_checks,
        extension::create_custom_check,
        extension::update_custom_check,
        extension::delete_custom_check,
        extension::list_custom_extractors,
        extension::create_custom_extractor,
        extension::update_custom_extractor,
        extension::delete_custom_extractor,
    ];
    s
}
