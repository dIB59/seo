use crate::repository::RepositoryResult;
use async_trait::async_trait;
use sqlx::SqlitePool;


pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Upsert a `(key, value)` row into the `app_kv_settings` KV store.
    /// Centralized so the "no column for this key" path and the
    /// "structured INSERT failed because the column doesn't exist yet"
    /// fallback path share one INSERT body — previously the same upsert
    /// was hand-written in two places and could drift on the
    /// `updated_at = datetime('now')` clause.
    async fn upsert_kv(&self, key: &str, value: &str) -> RepositoryResult<()> {
        sqlx::query(
            "INSERT INTO app_kv_settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Returns the column name in the structured `settings` table for `k`, or
    /// `None` if the key belongs in the `app_kv_settings` KV store.
    /// Note: both `gemini_api_key` and its alias `google_api_key` map to the
    /// same column here, so the previous separate `canonical_key` aliasing
    /// step is unnecessary — this map is the single source of truth for
    /// alias resolution.
    fn column_for_key(k: &str) -> Option<&'static str> {
        match k {
            "openai_api_key"       => Some("openai_api_key"),
            "anthropic_api_key"    => Some("anthropic_api_key"),
            "gemini_api_key"
            | "google_api_key"     => Some("google_api_key"),
            "default_ai_provider"  => Some("default_ai_provider"),
            "default_max_pages"    => Some("default_max_pages"),
            "default_max_depth"    => Some("default_max_depth"),
            "default_rate_limit_ms"=> Some("default_rate_limit_ms"),
            "theme"                => Some("theme"),
            "gemini_enabled"       => Some("gemini_enabled"),
            "gemini_persona"       => Some("gemini_persona"),
            "gemini_requirements"  => Some("gemini_requirements"),
            "gemini_context_options" => Some("gemini_context_options"),
            "gemini_prompt_blocks" => Some("gemini_prompt_blocks"),
            "signed_license"       => Some("signed_license"),
            _                      => None,
        }
    }
}

#[async_trait]
impl crate::repository::SettingsRepository for SettingsRepository {
    async fn get_setting(&self, key: &str) -> RepositoryResult<Option<String>> {
        let Some(column) = SettingsRepository::column_for_key(key) else {
            let v = sqlx::query_scalar::<_, String>(
                "SELECT value FROM app_kv_settings WHERE key = ?",
            )
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
            return Ok(v);
        };

        let query = format!("SELECT {} FROM settings WHERE id = 1", column);
        // Use Option<String> so that NULL column values are returned as None
        // rather than causing a decode error.
        let result = sqlx::query_scalar::<_, Option<String>>(&query)
            .fetch_optional(&self.pool)
            .await;

        match result {
            // Row found; the inner Option<String> handles NULL column values.
            Ok(Some(inner)) => Ok(inner),
            // No row at id=1 yet.
            Ok(None) => Ok(None),
            Err(e) => {
                let msg_lower = e.to_string().to_lowercase();
                if msg_lower.contains("no such column") || msg_lower.contains("no column named") {
                    // Fallback to KV store for columns not yet in the structured table
                    let v = sqlx::query_scalar::<_, String>(
                        "SELECT value FROM app_kv_settings WHERE key = ?",
                    )
                    .bind(key)
                    .fetch_optional(&self.pool)
                    .await?;
                    return Ok(v);
                }
                Err(e.into())
            }
        }
    }

    async fn set_setting(&self, key: &str, value: &str) -> RepositoryResult<()> {
        tracing::debug!("Updating setting: {}", key);

        let Some(column) = SettingsRepository::column_for_key(key) else {
            return self.upsert_kv(key, value).await;
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
                let msg_lower = e.to_string().to_lowercase();
                if msg_lower.contains("no such column")
                    || msg_lower.contains("no column named")
                    || msg_lower.contains("check constraint failed")
                {
                    // Fallback to KV store for columns not yet in the structured table
                    return self.upsert_kv(key, value).await;
                }
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::SettingsRepository as _;
    use crate::test_utils::fixtures;

    #[tokio::test]
    async fn test_get_set_setting_key_value() {
        let pool = fixtures::setup_test_db().await;
        let repo = super::SettingsRepository::new(pool.clone());

        // Ensure no key exists initially
        let v = repo.get_setting("gemini_prompt_blocks").await.unwrap();
        assert!(v.is_none());

        // Set and get
        repo.set_setting("gemini_prompt_blocks", "[]").await.unwrap();
        let v = repo.get_setting("gemini_prompt_blocks").await.unwrap();
        assert_eq!(v.unwrap(), "[]");

        // Alias key mapping
        repo.set_setting("google_api_key", "gkey").await.unwrap();
        let v = repo.get_setting("gemini_api_key").await.unwrap();
        assert_eq!(v.unwrap(), "gkey");
    }

    /// Regression test: `ai_source` is a dynamic key not in the structured settings
    /// table. Before migration 0032 (app_kv_settings), saving this key would fail
    /// because the KV SQL targeted the structured single-row `settings` table which
    /// has no `key` / `value` columns.
    #[tokio::test]
    async fn test_ai_source_set_and_get() {
        let pool = fixtures::setup_test_db().await;
        let repo = super::SettingsRepository::new(pool.clone());

        // Default: no value stored yet
        let v = repo.get_setting("ai_source").await.unwrap();
        assert!(v.is_none(), "ai_source should be absent initially");

        // Save "local" — this is the operation that previously failed
        repo.set_setting("ai_source", "local")
            .await
            .expect("set_setting(ai_source, local) must not fail");

        let v = repo.get_setting("ai_source").await.unwrap();
        assert_eq!(v.unwrap(), "local");

        // Overwrite with "gemini"
        repo.set_setting("ai_source", "gemini")
            .await
            .expect("set_setting(ai_source, gemini) must not fail");

        let v = repo.get_setting("ai_source").await.unwrap();
        assert_eq!(v.unwrap(), "gemini");
    }

    /// Companion regression: `local_model_active_id` uses the same KV path.
    #[tokio::test]
    async fn test_local_model_active_id_set_and_get() {
        let pool = fixtures::setup_test_db().await;
        let repo = super::SettingsRepository::new(pool.clone());

        let v = repo.get_setting("local_model_active_id").await.unwrap();
        assert!(v.is_none());

        repo.set_setting("local_model_active_id", "llama3-8b-q4")
            .await
            .expect("set_setting(local_model_active_id) must not fail");

        let v = repo.get_setting("local_model_active_id").await.unwrap();
        assert_eq!(v.unwrap(), "llama3-8b-q4");
    }
}
