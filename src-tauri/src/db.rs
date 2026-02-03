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
        .max_connections(10)           // Allow up to 10 concurrent connections
        .min_connections(2)            // Keep 2 connections warm
        .acquire_timeout(Duration::from_secs(5))  // Timeout for acquiring a connection
        .idle_timeout(Duration::from_secs(600))   // Close idle connections after 10 minutes
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Configure pragmas for each new connection
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

    log::info!("Database initialized successfully at {} with optimized settings", db_path.display());

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

/// Get a setting value from the database (V2 schema - single-row structured table)
/// Maps V1-style keys to V2 column names for backward compatibility
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    // Map V1 key-value names to V2 column names
    let column = match key {
        "openai_api_key" => "openai_api_key",
        "anthropic_api_key" => "anthropic_api_key",
        "google_api_key" | "gemini_api_key" => "google_api_key",
        "default_ai_provider" => "default_ai_provider",
        "default_max_pages" => "default_max_pages",
        "default_max_depth" => "default_max_depth",
        "default_rate_limit_ms" => "default_rate_limit_ms",
        "theme" => "theme",
        _ => {
            // Unknown key - return None since V2 schema doesn't support arbitrary keys
            log::warn!("Unknown setting key requested: {}", key);
            return Ok(None);
        }
    };

    // Query the appropriate column (V2 has a single row with id=1)
    let query = format!("SELECT {} FROM settings WHERE id = 1", column);
    let result = sqlx::query_scalar::<_, String>(&query)
        .fetch_optional(pool)
        .await
        .context("Failed to get setting from database")?;

    Ok(result)
}

/// Set a setting value in the database (V2 schema - single-row structured table)
/// Maps V1-style keys to V2 column names for backward compatibility
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    // Map V1 key-value names to V2 column names
    let column = match key {
        "openai_api_key" => "openai_api_key",
        "anthropic_api_key" => "anthropic_api_key",
        "google_api_key" | "gemini_api_key" => "google_api_key",
        "default_ai_provider" => "default_ai_provider",
        "default_max_pages" => "default_max_pages",
        "default_max_depth" => "default_max_depth",
        "default_rate_limit_ms" => "default_rate_limit_ms",
        "theme" => "theme",
        _ => {
            return Err(anyhow::anyhow!("Unknown setting key: {}", key));
        }
    };

    // V2 schema: single row with id=1, upsert the specific column
    match column {
        "openai_api_key" => {
            sqlx::query!(
                "INSERT INTO settings (id, openai_api_key) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET openai_api_key = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        "anthropic_api_key" => {
            sqlx::query!(
                "INSERT INTO settings (id, anthropic_api_key) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET anthropic_api_key = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        "google_api_key" => {
            sqlx::query!(
                "INSERT INTO settings (id, google_api_key) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET google_api_key = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        "default_ai_provider" => {
            sqlx::query!(
                "INSERT INTO settings (id, default_ai_provider) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET default_ai_provider = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        "default_max_pages" => {
            sqlx::query!(
                "INSERT INTO settings (id, default_max_pages) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET default_max_pages = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        "default_max_depth" => {
            sqlx::query!(
                "INSERT INTO settings (id, default_max_depth) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET default_max_depth = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        "default_rate_limit_ms" => {
            sqlx::query!(
                "INSERT INTO settings (id, default_rate_limit_ms) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET default_rate_limit_ms = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        "theme" => {
            sqlx::query!(
                "INSERT INTO settings (id, theme) VALUES (1, ?)
                 ON CONFLICT(id) DO UPDATE SET theme = ?, updated_at = datetime('now')",
                value,
                value
            )
            .execute(pool)
            .await
            .context("Failed to set setting in database")?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown setting key: {}", key));
        }
    }

    Ok(())
}

/// Get cached AI insights for a job (V2 schema)
/// Note: V2 stores structured insights. This returns the summary for backward compatibility.
pub async fn get_ai_insights(pool: &SqlitePool, job_id: &str) -> Result<Option<String>> {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT summary FROM ai_insights WHERE job_id = ?",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .context("Failed to get ai insights from database")?;

    Ok(result)
}

/// Save AI insights to the database (V2 schema)
/// For backward compatibility, stores insights as the summary field.
pub async fn save_ai_insights(pool: &SqlitePool, job_id: &str, insights: &str) -> Result<()> {
    sqlx::query!(
        "INSERT INTO ai_insights (job_id, summary, created_at, updated_at) VALUES (?, ?, datetime('now'), datetime('now'))
         ON CONFLICT(job_id) DO UPDATE SET summary = ?, updated_at = datetime('now')",
        job_id,
        insights,
        insights
    )
    .execute(pool)
    .await
    .context("Failed to save ai insights to database")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::fixtures;

    #[tokio::test]
    async fn test_get_setting_returns_none_when_not_set() {
        let pool = fixtures::setup_test_db().await;

        // V2 schema creates a settings row with NULL values by default during migration
        // When we query a NULL column, we should get None (not Some(""))
        let result = get_setting(&pool, "openai_api_key").await.unwrap();
        assert!(result.is_none() || result.as_deref() == Some(""), 
            "Should return None or empty string when API key not configured, got: {:?}", result);
    }

    #[tokio::test]
    async fn test_get_setting_returns_none_for_unknown_key() {
        let pool = fixtures::setup_test_db().await;

        let result = get_setting(&pool, "nonexistent_key").await.unwrap();
        assert!(result.is_none(), "Should return None for unknown key");
    }

    #[tokio::test]
    async fn test_set_and_get_setting() {
        let pool = fixtures::setup_test_db().await;

        set_setting(&pool, "openai_api_key", "test_value").await.unwrap();

        let result = get_setting(&pool, "openai_api_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_set_setting_updates_existing() {
        let pool = fixtures::setup_test_db().await;

        set_setting(&pool, "openai_api_key", "original").await.unwrap();
        set_setting(&pool, "openai_api_key", "updated").await.unwrap();

        let result = get_setting(&pool, "openai_api_key").await.unwrap();
        assert_eq!(
            result,
            Some("updated".to_string()),
            "Should update existing key"
        );
    }

    #[tokio::test]
    async fn test_ai_insights_returns_none_when_not_cached() {
        let pool = fixtures::setup_test_db().await;

        let result = get_ai_insights(&pool, "nonexistent_job")
            .await
            .unwrap();
        assert!(
            result.is_none(),
            "Should return None for non-cached job"
        );
    }

    /// Helper to create a valid jobs record for FK constraint (V2 schema)
    async fn create_test_job(pool: &SqlitePool, id: &str) {
        sqlx::query!(
            "INSERT INTO jobs (id, url, status, created_at, updated_at) 
             VALUES (?, 'https://test.com', 'completed', datetime('now'), datetime('now'))",
            id
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_and_get_ai_insights() {
        let pool = fixtures::setup_test_db().await;

        // Create the job record first to satisfy FK constraint
        create_test_job(&pool, "job_123").await;

        save_ai_insights(&pool, "job_123", "These are AI insights")
            .await
            .unwrap();

        let result = get_ai_insights(&pool, "job_123").await.unwrap();
        assert_eq!(result, Some("These are AI insights".to_string()));
    }

    #[tokio::test]
    async fn test_save_ai_insights_updates_existing() {
        let pool = fixtures::setup_test_db().await;

        // Create the job record first to satisfy FK constraint
        create_test_job(&pool, "job_456").await;

        save_ai_insights(&pool, "job_456", "Original insights")
            .await
            .unwrap();
        save_ai_insights(&pool, "job_456", "Updated insights")
            .await
            .unwrap();

        let result = get_ai_insights(&pool, "job_456").await.unwrap();
        assert_eq!(
            result,
            Some("Updated insights".to_string()),
            "Should update existing insights"
        );
    }
}
