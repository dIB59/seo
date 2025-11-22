// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

use crate::db::DbState;

mod analysis;
mod db;
mod error;
mod extractor;
mod taskqueue;

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
            app.manage(DbState(pool));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            analysis::start_analysis,
            analysis::get_analysis_progress,
            // get_analysis_list,
            // get_analysis,
            // pause_analysis,
            // resume_analysis,
            // delete_analysis,
            // export_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
