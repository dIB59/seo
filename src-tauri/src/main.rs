// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

use crate::db::DbState;

mod analysis;
mod application;
mod db;
mod domain;
mod error;
mod extractor;
mod repository;
mod service;

//TODO:
//-implement broken links, if internal link returns 404 it is broken
//-implement pagination for get all jobs

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    tauri::Builder::default()
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
