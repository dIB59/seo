-- Migration to add 'discovering' status to analysis_results and analysis_jobs

-- 1. Update analysis_results table
DROP VIEW IF EXISTS v_analysis_overview;
DROP VIEW IF EXISTS v_job_progress;
DROP VIEW IF EXISTS v_analysis_jobs;
DROP VIEW IF EXISTS v_page_details;

CREATE TABLE analysis_results_new (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('analyzing', 'completed', 'error', 'paused', 'discovering')),
    progress REAL NOT NULL DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    total_pages INTEGER NOT NULL DEFAULT 0,
    analyzed_pages INTEGER NOT NULL DEFAULT 0,
    started_at DATETIME,
    completed_at DATETIME,
    sitemap_found BOOLEAN NOT NULL DEFAULT 0,
    robots_txt_found BOOLEAN NOT NULL DEFAULT 0,
    ssl_certificate BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK (analyzed_pages <= total_pages),
    CHECK (completed_at IS NULL OR completed_at >= started_at)
);

INSERT INTO analysis_results_new 
SELECT id, url, status, progress, total_pages, analyzed_pages, 
       started_at, completed_at, sitemap_found, robots_txt_found, 
       ssl_certificate, created_at, updated_at
FROM analysis_results;

DROP TABLE analysis_results;
ALTER TABLE analysis_results_new RENAME TO analysis_results;

CREATE INDEX ix_analysis_results_created_at ON analysis_results(created_at);
CREATE INDEX ix_analysis_results_status ON analysis_results(status);
CREATE INDEX ix_analysis_results_url ON analysis_results(url);

-- 2. Update analysis_jobs table
CREATE TABLE analysis_jobs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    settings_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'processing', 'completed', 'failed', 'discovering')),
    result_id TEXT,
    FOREIGN KEY (settings_id) REFERENCES analysis_settings(id) ON DELETE CASCADE,
    FOREIGN KEY (result_id) REFERENCES analysis_results(id) ON DELETE SET NULL
);

INSERT INTO analysis_jobs_new (id, url, settings_id, created_at, updated_at, status, result_id)
SELECT id, url, settings_id, created_at, updated_at, status, result_id
FROM analysis_jobs;

DROP TABLE analysis_jobs;
ALTER TABLE analysis_jobs_new RENAME TO analysis_jobs;

CREATE INDEX ix_analysis_jobs_created_at ON analysis_jobs(created_at);
CREATE INDEX ix_analysis_jobs_result_id ON analysis_jobs(result_id);
CREATE INDEX ix_analysis_jobs_status ON analysis_jobs(status);

-- 3. Recreate views
CREATE VIEW v_job_progress AS
SELECT 
    aj.id as job_id,
    aj.url,
    aj.status as job_status,
    aj.created_at as job_created_at,
    aj.updated_at as job_updated_at,
    aj.result_id,
    ar.status as analysis_status,
    ar.progress,
    ar.analyzed_pages,
    ar.total_pages,
    ar.started_at,
    ar.completed_at
FROM analysis_jobs aj
LEFT JOIN analysis_results ar ON aj.result_id = ar.id;

CREATE VIEW v_analysis_overview AS
SELECT 
    ar.id,
    ar.url,
    ar.status,
    ar.progress,
    ar.analyzed_pages,
    ar.total_pages,
    ar.started_at,
    ar.completed_at,
    ar.created_at,
    ar.updated_at
FROM analysis_results ar;

CREATE VIEW v_analysis_jobs AS
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

CREATE VIEW v_page_details AS
SELECT 
    pa.id,
    pa.analysis_id,
    pa.url,
    pa.title,
    pa.status_code,
    pa.load_time,
    pa.word_count,
    pa.mobile_friendly,
    COUNT(CASE WHEN si.type = 'critical' THEN 1 END) as critical_issues,
    COUNT(CASE WHEN si.type = 'warning' THEN 1 END) as warning_issues,
    COUNT(CASE WHEN si.type = 'suggestion' THEN 1 END) as suggestion_issues
FROM page_analysis pa
LEFT JOIN seo_issues si ON pa.id = si.page_id
GROUP BY pa.id;
