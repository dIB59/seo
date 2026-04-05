-- =============================================================================
-- Migration 0030 DOWN: Recreate extension tables
-- =============================================================================
-- Restores the tables dropped by the up migration, matching the schema from
-- 0028_create_extension_tables.up.sql.
-- =============================================================================

CREATE TABLE IF NOT EXISTS audit_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'seo',
    severity TEXT NOT NULL DEFAULT 'warning',
    description TEXT,
    rule_type TEXT NOT NULL,
    target_field TEXT NOT NULL,
    condition TEXT,
    threshold_value TEXT,
    regex_pattern TEXT,
    recommendation TEXT,
    learn_more_url TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(name)
);

CREATE TABLE IF NOT EXISTS extractor_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    extractor_type TEXT NOT NULL DEFAULT 'css_selector',
    selector TEXT NOT NULL,
    attribute TEXT,
    post_process TEXT,
    storage_type TEXT NOT NULL DEFAULT 'json',
    target_column TEXT,
    target_table TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (storage_type IN ('column', 'json', 'separate_table'))
);

CREATE TABLE IF NOT EXISTS page_extensions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    extractor_id TEXT NOT NULL REFERENCES extractor_configs(id) ON DELETE CASCADE,
    value_text TEXT,
    value_number REAL,
    value_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(page_id, extractor_id)
);

CREATE TABLE IF NOT EXISTS page_keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    keyword TEXT NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    density REAL,
    is_meta_keyword INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(page_id, keyword)
);

CREATE TABLE IF NOT EXISTS page_href_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    rel TEXT NOT NULL,
    href TEXT NOT NULL,
    type TEXT,
    sizes TEXT,
    media TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(page_id, rel, href)
);

CREATE TABLE IF NOT EXISTS audit_checks (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'seo',
    check_type TEXT NOT NULL DEFAULT 'selector_count',
    selector TEXT,
    field TEXT,
    condition TEXT NOT NULL DEFAULT '{}',
    weight REAL NOT NULL DEFAULT 1.0,
    pass_score REAL NOT NULL DEFAULT 1.0,
    fail_score REAL NOT NULL DEFAULT 0.0,
    pass_message TEXT,
    fail_message TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (weight >= 0.0 AND weight <= 2.0)
);

CREATE INDEX IF NOT EXISTS idx_audit_rules_category ON audit_rules(category);
CREATE INDEX IF NOT EXISTS idx_audit_rules_enabled ON audit_rules(is_enabled);
CREATE INDEX IF NOT EXISTS idx_extractor_configs_enabled ON extractor_configs(is_enabled);
CREATE INDEX IF NOT EXISTS idx_page_extensions_page_id ON page_extensions(page_id);
CREATE INDEX IF NOT EXISTS idx_page_keywords_page_id ON page_keywords(page_id);
CREATE INDEX IF NOT EXISTS idx_page_keywords_keyword ON page_keywords(keyword);
CREATE INDEX IF NOT EXISTS idx_page_href_tags_page_id ON page_href_tags(page_id);
CREATE INDEX IF NOT EXISTS idx_audit_checks_category ON audit_checks(category);
CREATE INDEX IF NOT EXISTS idx_audit_checks_enabled ON audit_checks(is_enabled);
