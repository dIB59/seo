// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

use crate::db::DbState;

mod analysis;
mod application;
mod commands;
mod db;
mod domain;
mod error;
mod extractor;
mod gemini;
mod repository;
mod service;

//TODO:
//-implement pagination for get all jobs
//-remove AnalysisStatus from result, is should just be job status, maybe pages should have one
//-create custom issue rules to define what an issue is
//-search in pages table
//-create proper report
//-add pausing job
//-fix when app quits while job is being processed
//-Explain what the elements in the issues are
//-add abiity to delete job
//-when on page, you can press on the link to go to that page, that does not work
//-xml file path not found due to redirection

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // block on async init so the pool is available before commands run
            let pool = tauri::async_runtime::block_on(async {
                db::init_db(app.handle())
                    .await
                    .unwrap_or_else(|e| panic!("failed to init db: {}", e))
            });

            let processor = std::sync::Arc::new(application::JobProcessor::new(pool.clone()));
            let proc_clone = processor.clone();
            tauri::async_runtime::spawn(async move {
                proc_clone.run().await.expect("job-processor died")
            });

            app.manage(DbState(pool));
            app.manage(processor);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            analysis::start_analysis,
            analysis::get_analysis_progress,
            analysis::get_all_jobs,
            analysis::cancel_analysis,
            analysis::get_result,
            commands::get_gemini_insights,
            commands::get_gemini_api_key,
            commands::set_gemini_api_key,
            commands::get_gemini_persona,
            commands::set_gemini_persona,
            commands::get_gemini_requirements,
            commands::set_gemini_requirements,
            commands::get_gemini_context_options,
            commands::set_gemini_context_options,
            commands::get_gemini_prompt_blocks,
            commands::set_gemini_prompt_blocks,
            commands::get_gemini_enabled,
            commands::set_gemini_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
