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

/// Get a setting value from the database
pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let result = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .context("Failed to get setting from database")?;
    
    Ok(result)
}

/// Set a setting value in the database
pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = CURRENT_TIMESTAMP"
    )
    .bind(key)
    .bind(value)
    .bind(value)
    .execute(pool)
    .await
    .context("Failed to set setting in database")?;
    
    Ok(())
}

/// Get cached AI insights for an analysis
pub async fn get_ai_insights(pool: &SqlitePool, analysis_id: &str) -> Result<Option<String>> {
    let result = sqlx::query_scalar::<_, String>("SELECT insights FROM analysis_ai_insights WHERE analysis_id = ?")
        .bind(analysis_id)
        .fetch_optional(pool)
        .await
        .context("Failed to get ai insights from database")?;
    
    Ok(result)
}

/// Save AI insights to the database
pub async fn save_ai_insights(pool: &SqlitePool, analysis_id: &str, insights: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO analysis_ai_insights (analysis_id, insights, created_at) VALUES (?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(analysis_id) DO UPDATE SET insights = ?, created_at = CURRENT_TIMESTAMP"
    )
    .bind(analysis_id)
    .bind(insights)
    .bind(insights)
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
        
        let result = get_setting(&pool, "nonexistent_key").await.unwrap();
        assert!(result.is_none(), "Should return None for non-existent key");
    }

    #[tokio::test]
    async fn test_set_and_get_setting() {
        let pool = fixtures::setup_test_db().await;
        
        set_setting(&pool, "test_key", "test_value").await.unwrap();
        
        let result = get_setting(&pool, "test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_set_setting_updates_existing() {
        let pool = fixtures::setup_test_db().await;
        
        set_setting(&pool, "update_key", "original").await.unwrap();
        set_setting(&pool, "update_key", "updated").await.unwrap();
        
        let result = get_setting(&pool, "update_key").await.unwrap();
        assert_eq!(result, Some("updated".to_string()), "Should update existing key");
    }

    #[tokio::test]
    async fn test_ai_insights_returns_none_when_not_cached() {
        let pool = fixtures::setup_test_db().await;
        
        let result = get_ai_insights(&pool, "nonexistent_analysis").await.unwrap();
        assert!(result.is_none(), "Should return None for non-cached analysis");
    }

    /// Helper to create a valid analysis_results record for FK constraint
    async fn create_test_analysis(pool: &SqlitePool, id: &str) {
        sqlx::query(
            "INSERT INTO analysis_results (id, url, status, progress, analyzed_pages, total_pages, sitemap_found, robots_txt_found, ssl_certificate) 
             VALUES (?, 'https://test.com', 'completed', 100.0, 1, 1, 0, 0, 1)"
        )
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_and_get_ai_insights() {
        let pool = fixtures::setup_test_db().await;
        
        // Create the analysis record first to satisfy FK constraint
        create_test_analysis(&pool, "analysis_123").await;
        
        save_ai_insights(&pool, "analysis_123", "These are AI insights").await.unwrap();
        
        let result = get_ai_insights(&pool, "analysis_123").await.unwrap();
        assert_eq!(result, Some("These are AI insights".to_string()));
    }

    #[tokio::test]
    async fn test_save_ai_insights_updates_existing() {
        let pool = fixtures::setup_test_db().await;
        
        // Create the analysis record first to satisfy FK constraint
        create_test_analysis(&pool, "analysis_456").await;
        
        save_ai_insights(&pool, "analysis_456", "Original insights").await.unwrap();
        save_ai_insights(&pool, "analysis_456", "Updated insights").await.unwrap();
        
        let result = get_ai_insights(&pool, "analysis_456").await.unwrap();
        assert_eq!(result, Some("Updated insights".to_string()), "Should update existing insights");
    }
}
