PRAGMA foreign_keys = OFF;

-- 1. Drop dependent views and triggers
DROP VIEW IF EXISTS v_job_summary;
DROP TRIGGER IF EXISTS trg_update_job_stats_on_page_insert;
DROP TRIGGER IF EXISTS trg_update_job_stats_on_issue_insert;


-- 2. Create new table with updated schema
CREATE TABLE jobs_new (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    -- Update: Replaced 'running' with 'discovery', 'processing'
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'discovery', 'processing', 'completed', 'failed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    
    -- Settings
    max_pages INTEGER NOT NULL DEFAULT 100,
    max_depth INTEGER NOT NULL DEFAULT 3,
    respect_robots_txt INTEGER NOT NULL DEFAULT 1,
    include_subdomains INTEGER NOT NULL DEFAULT 0,
    rate_limit_ms INTEGER NOT NULL DEFAULT 1000,
    user_agent TEXT,
    
    -- Summary stats
    total_pages INTEGER NOT NULL DEFAULT 0,
    pages_crawled INTEGER NOT NULL DEFAULT 0,
    total_issues INTEGER NOT NULL DEFAULT 0,
    critical_issues INTEGER NOT NULL DEFAULT 0,
    warning_issues INTEGER NOT NULL DEFAULT 0,
    info_issues INTEGER NOT NULL DEFAULT 0,
    
    -- Progress tracking
    progress REAL NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 100.0),
    -- Removed: current_stage
    
    -- New column from 0020 (retained)
    lighthouse_analysis INTEGER NOT NULL DEFAULT 0,
    
    error_message TEXT
);

-- 3. Copy data
INSERT INTO jobs_new (
    id, url, status, created_at, updated_at, completed_at,
    max_pages, max_depth, respect_robots_txt, include_subdomains, rate_limit_ms, user_agent,
    total_pages, pages_crawled, total_issues, critical_issues, warning_issues, info_issues,
    progress, lighthouse_analysis, error_message
)
SELECT 
    id, url, 
    CASE status 
        WHEN 'running' THEN 'processing' -- Migrate legacy status
        ELSE status 
    END,
    created_at, updated_at, completed_at,
    max_pages, max_depth, respect_robots_txt, include_subdomains, rate_limit_ms, user_agent,
    total_pages, pages_crawled, total_issues, critical_issues, warning_issues, info_issues,
    progress, lighthouse_analysis, error_message
FROM jobs;

-- 4. Swap tables
DROP TABLE jobs;
ALTER TABLE jobs_new RENAME TO jobs;

-- 5. Recreate Indexes
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_jobs_url ON jobs(url);

-- 6. Recreate Triggers
CREATE TRIGGER IF NOT EXISTS trg_jobs_updated_at
AFTER UPDATE ON jobs
BEGIN
    UPDATE jobs SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_update_job_stats_on_page_insert
AFTER INSERT ON pages
BEGIN
    UPDATE jobs SET 
        total_pages = (SELECT COUNT(*) FROM pages WHERE job_id = NEW.job_id),
        pages_crawled = (SELECT COUNT(*) FROM pages WHERE job_id = NEW.job_id)
    WHERE id = NEW.job_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_update_job_stats_on_issue_insert
AFTER INSERT ON issues
BEGIN
    UPDATE jobs SET 
        total_issues = (SELECT COUNT(*) FROM issues WHERE job_id = NEW.job_id),
        critical_issues = (SELECT COUNT(*) FROM issues WHERE job_id = NEW.job_id AND severity = 'critical'),
        warning_issues = (SELECT COUNT(*) FROM issues WHERE job_id = NEW.job_id AND severity = 'warning'),
        info_issues = (SELECT COUNT(*) FROM issues WHERE job_id = NEW.job_id AND severity = 'info')
    WHERE id = NEW.job_id;
END;

-- 7. Recreate Views
CREATE VIEW IF NOT EXISTS v_job_summary AS
SELECT 
    j.*,
    (SELECT COUNT(*) FROM pages p WHERE p.job_id = j.id) as actual_page_count,
    (SELECT COUNT(*) FROM issues i WHERE i.job_id = j.id) as actual_issue_count,
    (SELECT AVG(pl.performance_score) FROM page_lighthouse pl 
     JOIN pages p ON p.id = pl.page_id WHERE p.job_id = j.id) as avg_performance_score
FROM jobs j;

PRAGMA foreign_keys = ON;
