use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get a setting value from the database (V2 schema - single-row structured table)
    /// Maps V1-style keys to V2 column names for backward compatibility
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
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
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get setting from database")?;

        Ok(result)
    }

    /// Set a setting value in the database (V2 schema - single-row structured table)
    /// Maps V1-style keys to V2 column names for backward compatibility
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
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
                .execute(&self.pool)
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
                .execute(&self.pool)
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
                .execute(&self.pool)
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
                .execute(&self.pool)
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
                .execute(&self.pool)
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
                .execute(&self.pool)
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
                .execute(&self.pool)
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
                .execute(&self.pool)
                .await
                .context("Failed to set setting in database")?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown setting key: {}", key));
            }
        }

        Ok(())
    }
}
