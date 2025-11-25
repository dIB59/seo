use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub struct DbState(pub SqlitePool);

use tauri::{AppHandle, Manager};

pub async fn init_db(app: &AppHandle) -> Result<SqlitePool> {
    // Get the app's data directory
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("failed to get app data dir")?;

    log::info!("App data directory: {}", app_data_dir.display());

    // Ensure the directory exists
    std::fs::create_dir_all(&app_data_dir).context(format!(
        "failed to create app data dir: {}",
        app_data_dir.display()
    ))?;

    // Verify directory was created and is writable
    if !app_data_dir.exists() {
        return Err(anyhow::anyhow!(
            "App data directory does not exist after creation"
        ));
    }

    // Create the database path
    let db_path = app_data_dir.join("analysisdev.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    log::info!("Database URL: {}", db_url);

    // Connect to the DB (will create file if using sqlite file URL)
    let pool = SqlitePool::connect(&db_url).await.context(format!(
        "failed to connect to database at {}",
        db_path.display()
    ))?;

    // Run embedded migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    log::info!("Database initialized successfully at {}", db_path.display());

    Ok(pool)
}
