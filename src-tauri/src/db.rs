use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::time::Duration;

pub struct DbState(pub SqlitePool);

use tauri::{AppHandle, Manager};

/// Configure SQLite pragmas for optimal performance.
/// These are set per-connection via the after_connect callback.
async fn configure_sqlite_pragmas(conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::Error> {
    use sqlx::Executor;

    // WAL mode: Allows concurrent reads during writes, 5-10x faster writes
    conn.execute("PRAGMA journal_mode = WAL").await?;

    // NORMAL synchronous: 2-3x faster writes while still being safe (data is synced at critical moments)
    conn.execute("PRAGMA synchronous = NORMAL").await?;

    // 64MB cache: Significantly improves read performance for large datasets
    // Negative value = KB, so -65536 = 64MB
    conn.execute("PRAGMA cache_size = -65536").await?;

    // Enable memory-mapped I/O for faster reads (256MB)
    conn.execute("PRAGMA mmap_size = 268435456").await?;

    // 5 second timeout for busy connections (prevents "database locked" errors)
    conn.execute("PRAGMA busy_timeout = 5000").await?;

    // Use memory for temp tables (faster sorting, joins)
    conn.execute("PRAGMA temp_store = MEMORY").await?;

    // Enable foreign key constraints
    conn.execute("PRAGMA foreign_keys = ON").await?;

    Ok(())
}

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

    // Connect with optimized pool configuration
    let pool = SqlitePoolOptions::new()
        .max_connections(10) // Allow up to 10 concurrent connections
        .min_connections(2) // Keep 2 connections warm
        .acquire_timeout(Duration::from_secs(5)) // Timeout for acquiring a connection
        .idle_timeout(Duration::from_secs(600)) // Close idle connections after 10 minutes
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                configure_sqlite_pragmas(conn).await?;
                Ok(())
            })
        })
        .connect(&db_url)
        .await
        .context(format!(
            "failed to connect to database at {}",
            db_path.display()
        ))?;

    // Run embedded migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    log::info!(
        "Database initialized successfully at {} with optimized settings",
        db_path.display()
    );

    // Dump schema for recreation / versioning
    let schema_path = app_data_dir.join("schema.sql");
    dump_schema(&pool, &schema_path)
        .await
        .context("failed to dump schema")?;

    log::info!("Schema dumped to {}", schema_path.display());

    Ok(pool)
}

use std::fs::File;
use std::io::Write;

async fn dump_schema(pool: &SqlitePool, output_path: &std::path::Path) -> anyhow::Result<()> {
    let rows = sqlx::query!(
        r#"
        SELECT sql
        FROM sqlite_master
        WHERE sql IS NOT NULL
          AND type IN ('table', 'index', 'trigger')
        ORDER BY type, name;
        "#
    )
    .fetch_all(pool)
    .await
    .context("failed to read sqlite_master")?;

    let mut file = File::create(output_path).context("failed to create schema.sql")?;

    writeln!(
        file,
        "-- Auto-generated SQLite schema\n-- DO NOT EDIT MANUALLY\n"
    )?;

    for row in rows {
        if let Some(sql) = row.sql {
            writeln!(file, "{};\n", sql)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
