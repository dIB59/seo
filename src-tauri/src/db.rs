use std::fs::File;
use std::io::Write;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

const PRAGMA_STATEMENTS: &[&str] = &[
    "PRAGMA journal_mode = WAL",
    "PRAGMA synchronous = NORMAL",
    "PRAGMA cache_size = -65536",
    "PRAGMA mmap_size = 268435456",
    "PRAGMA busy_timeout = 5000",
    "PRAGMA temp_store = MEMORY",
    "PRAGMA foreign_keys = ON",
];

async fn configure_sqlite_pragmas(conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::Error> {
    use sqlx::Executor;
    for pragma in PRAGMA_STATEMENTS {
        conn.execute(*pragma).await?;
    }
    Ok(())
}

pub async fn init_db(app: &AppHandle) -> Result<SqlitePool> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("failed to get app data directory")?;

    tracing::info!("App data directory: {}", app_data_dir.display());

    std::fs::create_dir_all(&app_data_dir)
        .with_context(|| format!("failed to create app data directory: {}", app_data_dir.display()))?;

    let db_path = app_data_dir.join("analysisdev.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    tracing::info!("Database URL: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                configure_sqlite_pragmas(conn).await?;
                Ok(())
            })
        })
        .connect(&db_url)
        .await
        .with_context(|| format!("failed to connect to database at {}", db_path.display()))?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    tracing::info!(
        "Database initialized at {} with optimized settings",
        db_path.display()
    );

    let schema_path = app_data_dir.join("schema.sql");
    dump_schema(&pool, &schema_path)
        .await
        .context("failed to dump schema")?;

    tracing::info!("Schema dumped to {}", schema_path.display());

    Ok(pool)
}

async fn dump_schema(pool: &SqlitePool, output_path: &std::path::Path) -> Result<()> {
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

    let mut file = File::create(output_path)
        .with_context(|| format!("failed to create schema file: {}", output_path.display()))?;

    writeln!(file, "-- Auto-generated SQLite schema\n-- DO NOT EDIT MANUALLY\n")?;

    for row in rows {
        if let Some(sql) = row.sql {
            writeln!(file, "{};\n", sql)?;
            tracing::debug!("{:?}", sql);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
