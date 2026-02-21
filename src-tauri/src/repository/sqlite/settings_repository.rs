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

    fn canonical_key(key: &str) -> &str {
        match key {
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
                // Not in structured table, try KV table
                return sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
                    .bind(k)
                    .fetch_optional(&self.pool)
                    .await
                    .context("Failed to get setting from KV table");
            }
        };

        let query = format!("SELECT {} FROM settings WHERE id = 1", column);
        let result = sqlx::query_scalar::<_, String>(&query)
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(opt) => Ok(opt),
            Err(e) => {
                let e_msg = e.to_string();
                let msg_lower = e_msg.to_lowercase();
                if msg_lower.contains("no such column") || msg_lower.contains("no column named") {
                    // Try KV table fallback
                    return sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
                        .bind(k)
                        .fetch_optional(&self.pool)
                        .await
                        .context(format!("Failed to get setting from KV table after structured failure (structured error: {})", e_msg));
                }
                Err(e).context(format!(
                    "Failed to get setting from structured settings table. Inner error: {}",
                    e_msg
                ))
            }
        }
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let k = SettingsRepository::canonical_key(key);
        tracing::debug!("Updating setting: {} (canonical: {})", key, k);

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
                // Not in structured table, try KV table
                return sqlx::query(
                    "INSERT INTO settings (key, value) VALUES (?, ?)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
                )
                .bind(k)
                .bind(value)
                .execute(&self.pool)
                .await
                .map(|_| ())
                .context(format!("Failed to set setting '{}' in KV table", key));
            }
        };

        let query = format!(
            "INSERT INTO settings (id, {}) VALUES (1, ?)
             ON CONFLICT(id) DO UPDATE SET {} = ?, updated_at = datetime('now')",
            column, column
        );

        let res = sqlx::query(&query)
            .bind(value)
            .bind(value)
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => {
                tracing::debug!("Updated structured setting successfully: {}", column);
                Ok(())
            }
            Err(e) => {
                let e_msg = e.to_string();
                let msg_lower = e_msg.to_lowercase();
                if msg_lower.contains("no such column")
                    || msg_lower.contains("no column named")
                    || msg_lower.contains("check constraint failed")
                {
                    // Try KV table fallback
                    return sqlx::query(
                        "INSERT INTO settings (key, value) VALUES (?, ?)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
                    )
                    .bind(k)
                    .bind(value)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .context(format!("Failed to set setting '{}' in KV table after structured failure (structured error: {})", key, e_msg));
                }
                Err(e).context(format!(
                    "Failed to set setting '{}' in structured settings table. Inner error: {}",
                    key, e_msg
                ))
            }
        }
    }
}
