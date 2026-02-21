-- =============================================================================
-- Revert Migration 0023
-- =============================================================================

PRAGMA foreign_keys = OFF;

-- Drop dependent views
DROP VIEW IF EXISTS v_job_summary;

CREATE TABLE jobs_old (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    
    max_pages INTEGER NOT NULL DEFAULT 100,
    max_depth INTEGER NOT NULL DEFAULT 3,
    respect_robots_txt INTEGER NOT NULL DEFAULT 1,
    include_subdomains INTEGER NOT NULL DEFAULT 0,
    rate_limit_ms INTEGER NOT NULL DEFAULT 1000,
    user_agent TEXT,
    
    total_pages INTEGER NOT NULL DEFAULT 0,
    pages_crawled INTEGER NOT NULL DEFAULT 0,
    total_issues INTEGER NOT NULL DEFAULT 0,
    critical_issues INTEGER NOT NULL DEFAULT 0,
    warning_issues INTEGER NOT NULL DEFAULT 0,
    info_issues INTEGER NOT NULL DEFAULT 0,
    
    progress REAL NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 100.0),
    current_stage TEXT, -- Restored
    lighthouse_analysis INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);

INSERT INTO jobs_old (
    id, url, status, created_at, updated_at, completed_at,
    max_pages, max_depth, respect_robots_txt, include_subdomains, rate_limit_ms, user_agent,
    total_pages, pages_crawled, total_issues, critical_issues, warning_issues, info_issues,
    progress, current_stage, lighthouse_analysis, error_message
)
SELECT 
    id, url, 
    CASE status 
        WHEN 'discovery' THEN 'running'
        WHEN 'processing' THEN 'running'
        ELSE status 
    END,
    created_at, updated_at, completed_at,
    max_pages, max_depth, respect_robots_txt, include_subdomains, rate_limit_ms, user_agent,
    total_pages, pages_crawled, total_issues, critical_issues, warning_issues, info_issues,
    progress, NULL, lighthouse_analysis, error_message
FROM jobs;

DROP TABLE jobs;
ALTER TABLE jobs_old RENAME TO jobs;

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_jobs_url ON jobs(url);

CREATE TRIGGER IF NOT EXISTS trg_jobs_updated_at
AFTER UPDATE ON jobs
BEGIN
    UPDATE jobs SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Recreate Views
CREATE VIEW IF NOT EXISTS v_job_summary AS
SELECT 
    j.*,
    (SELECT COUNT(*) FROM pages p WHERE p.job_id = j.id) as actual_page_count,
    (SELECT COUNT(*) FROM issues i WHERE i.job_id = j.id) as actual_issue_count,
    (SELECT AVG(pl.performance_score) FROM page_lighthouse pl 
     JOIN pages p ON p.id = pl.page_id WHERE p.job_id = j.id) as avg_performance_score
FROM jobs j;

PRAGMA foreign_keys = ON;
