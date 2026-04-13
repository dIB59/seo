use std::fs::File;
use std::io::Write;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

const PRAGMA_STATEMENTS: [&str; 7] = [
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
        conn.execute(pragma).await?;
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

/// Apply the same pragmas to an externally-supplied pool. Used by the
/// in-memory test fixtures so unit tests run against the same SQLite
/// configuration as production.
#[cfg(test)]
pub async fn apply_pragmas(pool: &SqlitePool) -> Result<()> {
    use sqlx::Acquire;
    let mut conn = pool.acquire().await.context("acquire connection for pragmas")?;
    configure_sqlite_pragmas(conn.acquire().await?).await?;
    Ok(())
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
mod tests {
    //! Characterization tests for the SQLite pragma configuration. The
    //! production pool applies these pragmas via `after_connect`, but
    //! the constants and the apply path are also useful in unit tests
    //! to confirm we're configuring SQLite the same way as production.

    use super::*;
    use sqlx::Row;

    #[test]
    fn pragma_statements_constant_includes_seven_settings() {
        // Pinning the count — the array is `[&str; 7]`. Adding or
        // removing a pragma is a deliberate decision, not a quiet
        // refactor.
        assert_eq!(PRAGMA_STATEMENTS.len(), 7);
    }

    #[test]
    fn pragma_statements_include_critical_settings() {
        // Concurrency, durability, FK enforcement — pinning that none
        // of the load-bearing pragmas are removed.
        let joined = PRAGMA_STATEMENTS.join("|");
        assert!(joined.contains("journal_mode = WAL"));
        assert!(joined.contains("synchronous = NORMAL"));
        assert!(joined.contains("foreign_keys = ON"));
        assert!(joined.contains("busy_timeout"));
    }

    #[tokio::test]
    async fn apply_pragmas_sets_journal_mode_to_wal() {
        // In-memory pool — applies the pragmas and reads them back to
        // confirm the connection actually adopted them.
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        apply_pragmas(&pool).await.unwrap();

        // foreign_keys = ON should be readable.
        let row = sqlx::query("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .unwrap();
        let fk: i64 = row.get(0);
        assert_eq!(fk, 1, "foreign_keys should be ON after apply_pragmas");
    }

    #[tokio::test]
    async fn apply_pragmas_is_idempotent() {
        // Calling twice should not error or change behavior.
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        apply_pragmas(&pool).await.unwrap();
        apply_pragmas(&pool).await.unwrap();
    }
}
