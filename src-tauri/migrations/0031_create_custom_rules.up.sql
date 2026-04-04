-- custom_checks: user-defined SEO checks that produce issues based on extracted page data
CREATE TABLE IF NOT EXISTS custom_checks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'warning',
    -- condition: page field to evaluate
    field TEXT NOT NULL,
    -- operator: missing, lt, gt, contains, not_contains
    operator TEXT NOT NULL,
    -- threshold value (nullable for operators like 'missing')
    threshold TEXT,
    message_template TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- custom_extractors: user-defined CSS-selector-based data extractors
CREATE TABLE IF NOT EXISTS custom_extractors (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    -- output key in extracted_data
    key TEXT NOT NULL UNIQUE,
    selector TEXT NOT NULL,
    -- HTML attribute to read; NULL means text content
    attribute TEXT,
    multiple INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
