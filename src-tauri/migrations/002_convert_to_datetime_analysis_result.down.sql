PRAGMA foreign_keys = OFF;

-- Drop ALL dependent views first
DROP VIEW IF EXISTS v_analysis_overview;
DROP VIEW IF EXISTS v_analysis_jobs;

-- Revert DATETIME columns back to TEXT
CREATE TABLE analysis_results_old (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('analyzing', 'completed', 'error', 'paused')),
    progress REAL NOT NULL DEFAULT 0,
    total_pages INTEGER NOT NULL DEFAULT 0,
    analyzed_pages INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    sitemap_found BOOLEAN NOT NULL DEFAULT 0,
    robots_txt_found BOOLEAN NOT NULL DEFAULT 0,
    ssl_certificate BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Copy data back (SQLite auto-converts DATETIME to TEXT)
INSERT INTO analysis_results_old SELECT * FROM analysis_results;

-- Replace table
DROP TABLE analysis_results;
ALTER TABLE analysis_results_old RENAME TO analysis_results;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS ix_analysis_results_status ON analysis_results(status);
CREATE INDEX IF NOT EXISTS ix_analysis_results_url ON analysis_results(url);
CREATE INDEX IF NOT EXISTS ix_analysis_results_created_at ON analysis_results(created_at);

-- Recreate views at the END
CREATE VIEW IF NOT EXISTS v_analysis_overview AS
SELECT 
    ar.id,
    ar.url,
    ar.status,
    ar.progress,
    ar.analyzed_pages,
    ar.total_pages,
    ar.started_at,
    ar.completed_at,
    ai.critical,
    ai.warnings,
    ai.suggestions,
    assum.seo_score,
    assum.avg_load_time,
    assum.mobile_friendly_pages
FROM analysis_results ar
LEFT JOIN analysis_issues ai ON ar.id = ai.analysis_id
LEFT JOIN analysis_summary assum ON ar.id = assum.analysis_id;

CREATE VIEW IF NOT EXISTS v_analysis_jobs AS
SELECT 
    aj.id as job_id,
    aj.url,
    aj.status as job_status,
    aj.created_at as queued_at,
    ar.id as result_id,
    ar.status as analysis_status,
    ar.progress,
    ar.analyzed_pages,
    ar.total_pages,
    aset.max_pages,
    aset.lighthouse_analysis,
    aset.mobile_analysis
FROM analysis_jobs aj
LEFT JOIN analysis_results ar ON aj.result_id = ar.id
LEFT JOIN analysis_settings aset ON aj.settings_id = aset.id;

PRAGMA foreign_keys = ON;