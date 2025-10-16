CREATE TABLE IF NOT EXISTS analysis_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    max_pages INTEGER NOT NULL,
    include_external_links INTEGER NOT NULL,
    check_images INTEGER NOT NULL,
    mobile_analysis INTEGER NOT NULL,
    lighthouse_analysis INTEGER NOT NULL,
    delay_between_requests INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS analyses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    settings_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'queued',
    FOREIGN KEY (settings_id) REFERENCES analysis_settings(id) ON DELETE CASCADE
);
