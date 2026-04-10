-- Report templates: user-authored report structures with ordered sections.
-- Each template is a JSON array of TemplateSection variants.
-- One template is marked `is_active = 1` at a time.
CREATE TABLE IF NOT EXISTS report_templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    sections_json TEXT NOT NULL DEFAULT '[]',
    is_active INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed the default template. The sections_json is populated at
-- runtime by the app on first launch (AppState init checks for the
-- default template and inserts it if missing), because the JSON is
-- large and depends on Rust constants. This row is just the shell.
INSERT OR IGNORE INTO report_templates (id, name, is_builtin, is_active)
VALUES ('default', 'Default Report', 1, 1);
