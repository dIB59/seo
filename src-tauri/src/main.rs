// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::{commands, lifecycle};

fn main() {
    lifecycle::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(lifecycle::setup)
        .invoke_handler(tauri::generate_handler![
            commands::analysis::start_analysis,
            commands::analysis::get_analysis_progress,
            commands::analysis::get_all_jobs,
            commands::analysis::cancel_analysis,
            commands::analysis::get_result,
            commands::ai::get_gemini_insights,
            commands::ai::get_gemini_api_key,
            commands::ai::set_gemini_api_key,
            commands::ai::get_gemini_persona,
            commands::ai::set_gemini_persona,
            commands::ai::get_gemini_requirements,
            commands::ai::set_gemini_requirements,
            commands::ai::get_gemini_context_options,
            commands::ai::set_gemini_context_options,
            commands::ai::get_gemini_prompt_blocks,
            commands::ai::set_gemini_prompt_blocks,
            commands::ai::get_gemini_enabled,
            commands::ai::set_gemini_enabled,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(lifecycle::handle_run_event);
}
