use tauri_specta::{collect_commands, Commands};

mod ai;
pub mod analysis;
mod licensing;

pub fn register_commands() -> Commands<tauri::Wry> {
    let s = collect_commands![
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
        analysis::start_analysis,
        analysis::get_analysis_progress,
        analysis::get_all_jobs,
        analysis::get_paginated_jobs,
        analysis::cancel_analysis,
        analysis::get_result,
        licensing::activate_license,
        licensing::activate_with_key,
        licensing::get_license_tier,
    ];
    s
}
