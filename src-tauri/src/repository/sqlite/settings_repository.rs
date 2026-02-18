use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;

pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Canonicalize commonly-used keys (aliases) to the stored key name.
    fn canonical_key(key: &str) -> &str {
        match key {
            // In V2 schema the API key column is named `google_api_key`
            "google_api_key" | "gemini_api_key" => "google_api_key",
            "gemini_enabled" => "gemini_enabled",
            "gemini_persona" => "gemini_persona",
            "gemini_requirements" => "gemini_requirements",
            "gemini_context_options" => "gemini_context_options",
            "gemini_prompt_blocks" => "gemini_prompt_blocks",
            other => other,
        }
    }
}

#[async_trait]
impl crate::repository::SettingsRepository for SettingsRepository {
    async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let k = SettingsRepository::canonical_key(key);

        // Try key/value table first
        let kv_res = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
            .bind(k)
            .fetch_optional(&self.pool)
            .await;

        match kv_res {
            Ok(opt) => return Ok(opt),
            Err(e) => {
                // If column doesn't exist (schema uses structured table), fall back
                let msg = e.to_string();
                if !msg.contains("no column named") && !msg.contains("no such column") {
                    return Err(e).context("Failed to get setting from database")?;
                }
            }
        }

        // Fall back to structured single-row settings table
        let column = match k {
            "openai_api_key" => "openai_api_key",
            "anthropic_api_key" => "anthropic_api_key",
            "gemini_api_key" | "google_api_key" => "google_api_key",
            "default_ai_provider" => "default_ai_provider",
            "default_max_pages" => "default_max_pages",
            "default_max_depth" => "default_max_depth",
            "default_rate_limit_ms" => "default_rate_limit_ms",
            "theme" => "theme",
            "gemini_enabled" => "gemini_enabled",
            "gemini_persona" => "gemini_persona",
            "gemini_requirements" => "gemini_requirements",
            "gemini_context_options" => "gemini_context_options",
            "gemini_prompt_blocks" => "gemini_prompt_blocks",
            "signed_license" => "signed_license",
            _ => {
                tracing::warn!(
                    "Unknown setting key requested for structured table: {}",
                    key
                );
                return Ok(None);
            }
        };

        let query = format!("SELECT {} FROM settings WHERE id = 1", column);
        let result = sqlx::query_scalar::<_, String>(&query)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get setting from structured settings table")?;

        Ok(result)
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let k = SettingsRepository::canonical_key(key);
        tracing::debug!("Updating setting: {} (canonical: {})", key, k);

        // Try key/value upsert first (for compatibility with KV-style tables)
        let kv_res: std::result::Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> =
            sqlx::query(
                "INSERT INTO settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
            )
            .bind(k)
            .bind(value)
            .execute(&self.pool)
            .await;

        match kv_res {
            Ok(_) => {
                tracing::debug!("Updated setting via KV table: {}", k);
                return Ok(());
            }
            Err(e) => {
                let msg = e.to_string();
                if !msg.contains("no column named")
                    && !msg.contains("no such table")
                    && !msg.contains("no such column")
                {
                    tracing::error!("Failed to set setting in KV table: {}", e);
                    return Err(e)
                        .context(format!("Failed to set setting '{}' in database", key))?;
                }
                tracing::debug!("KV table update failed or table missing, falling back to structured table for: {}", k);
            }
        }

        // Fall back to structured single-row settings table
        let column = match k {
            "openai_api_key" => "openai_api_key",
            "anthropic_api_key" => "anthropic_api_key",
            "gemini_api_key" | "google_api_key" => "google_api_key",
            "default_ai_provider" => "default_ai_provider",
            "default_max_pages" => "default_max_pages",
            "default_max_depth" => "default_max_depth",
            "default_rate_limit_ms" => "default_rate_limit_ms",
            "theme" => "theme",
            "gemini_enabled" => "gemini_enabled",
            "gemini_persona" => "gemini_persona",
            "gemini_requirements" => "gemini_requirements",
            "gemini_context_options" => "gemini_context_options",
            "gemini_prompt_blocks" => "gemini_prompt_blocks",
            "signed_license" => "signed_license",
            _ => {
                let err_msg = format!("Unknown setting key: {}. This key must be added to the schema and SettingsRepository mapping.", key);
                tracing::error!("{}", err_msg);
                return Err(anyhow::anyhow!(err_msg));
            }
        };

        let query = format!(
            "INSERT INTO settings (id, {}) VALUES (1, ?)
             ON CONFLICT(id) DO UPDATE SET {} = ?, updated_at = datetime('now')",
            column, column
        );

        sqlx::query(&query)
            .bind(value)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update structured setting '{}': {}", column, e);
                e
            })
            .context(format!("Failed to set setting '{}' in structured settings table. Verify the column exists.", key))?;

        tracing::debug!("Updated structured setting successfully: {}", column);
        Ok(())
    }
}
