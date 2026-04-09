-- Migration 0032: Add generic KV table for dynamic settings
-- The `settings` table is a structured single-row table (id=1).
-- Dynamic keys like `ai_source` and `local_model_active_id` need a separate
-- key-value store that supports arbitrary keys.

CREATE TABLE IF NOT EXISTS app_kv_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
