#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::{commands, lifecycle};
use specta_typescript::BigIntExportBehavior;
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri_specta::Builder;

fn main() {
    lifecycle::init_logging();

    let builder = Builder::<tauri::Wry>::new()
        .commands(commands::register_commands())
        .events(tauri_specta::collect_events![
            app::service::processor::reporter::ProgressEvent,
            app::service::local_model::ModelDownloadEvent
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::default()
                .formatter(specta_typescript::formatter::prettier)
                .bigint(BigIntExportBehavior::Number),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    let invoke_handler = builder.invoke_handler();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            builder.mount_events(app);
            lifecycle::setup(app)
        })
        .invoke_handler(invoke_handler)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(lifecycle::handle_run_event);
}
