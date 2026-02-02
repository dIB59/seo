-- =============================================================================
-- Migration 0018 DOWN: Rollback Schema Redesign
-- =============================================================================
-- This restores the original schema structure
-- =============================================================================

-- Step 1: Drop triggers
DROP TRIGGER IF EXISTS trg_update_job_stats_on_issue_insert;
DROP TRIGGER IF EXISTS trg_update_job_stats_on_page_insert;
DROP TRIGGER IF EXISTS trg_ai_insights_updated_at;
DROP TRIGGER IF EXISTS trg_settings_updated_at;
DROP TRIGGER IF EXISTS trg_jobs_updated_at;

-- Step 2: Drop views
DROP VIEW IF EXISTS v_issues_by_severity;
DROP VIEW IF EXISTS v_page_with_issues;
DROP VIEW IF EXISTS v_job_summary;

-- Step 3: Drop new tables
DROP TABLE IF EXISTS page_images;
DROP TABLE IF EXISTS page_headings;
DROP TABLE IF EXISTS page_lighthouse;
DROP TABLE IF EXISTS ai_insights;
DROP TABLE IF EXISTS links;
DROP TABLE IF EXISTS issues;
DROP TABLE IF EXISTS pages;
DROP TABLE IF EXISTS jobs;
DROP TABLE IF EXISTS settings;

-- Step 4: Recreate original schema
-- -----------------------------------------------------------------------------

-- Original analysis_jobs table
CREATE TABLE analysis_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    progress REAL NOT NULL DEFAULT 0.0,
    current_stage TEXT,
    result_id TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

-- Original analysis_results table
CREATE TABLE analysis_results (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL REFERENCES analysis_jobs(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Original analysis_settings table
CREATE TABLE analysis_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL UNIQUE REFERENCES analysis_jobs(id) ON DELETE CASCADE,
    max_pages INTEGER NOT NULL DEFAULT 100,
    max_depth INTEGER NOT NULL DEFAULT 3,
    respect_robots_txt INTEGER NOT NULL DEFAULT 1,
    include_subdomains INTEGER NOT NULL DEFAULT 0,
    rate_limit_ms INTEGER NOT NULL DEFAULT 1000,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Original page_analysis table
CREATE TABLE page_analysis (
    id TEXT PRIMARY KEY NOT NULL,
    result_id TEXT NOT NULL REFERENCES analysis_results(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    depth INTEGER NOT NULL DEFAULT 0,
    status_code INTEGER,
    content_type TEXT,
    title TEXT,
    meta_description TEXT,
    canonical_url TEXT,
    robots_meta TEXT,
    word_count INTEGER,
    load_time_ms INTEGER,
    response_size_bytes INTEGER,
    headings TEXT,
    images TEXT,
    links_internal TEXT,
    links_external TEXT,
    lighthouse_performance REAL,
    lighthouse_accessibility REAL,
    lighthouse_best_practices REAL,
    lighthouse_seo REAL,
    lighthouse_fcp REAL,
    lighthouse_lcp REAL,
    lighthouse_tbt REAL,
    lighthouse_cls REAL,
    lighthouse_si REAL,
    lighthouse_tti REAL,
    lighthouse_details TEXT,
    crawled_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Original seo_issues table
CREATE TABLE seo_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_id TEXT NOT NULL REFERENCES analysis_results(id) ON DELETE CASCADE,
    page_id TEXT REFERENCES page_analysis(id) ON DELETE CASCADE,
    issue_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Original page_edge table
CREATE TABLE page_edge (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_page_id TEXT NOT NULL REFERENCES page_analysis(id) ON DELETE CASCADE,
    target_page_id TEXT REFERENCES page_analysis(id) ON DELETE SET NULL,
    link_text TEXT,
    link_type TEXT NOT NULL DEFAULT 'internal',
    is_followed INTEGER NOT NULL DEFAULT 1,
    UNIQUE (source_page_id, target_page_id)
);

-- Original analysis_summary table
CREATE TABLE analysis_summary (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_id TEXT NOT NULL UNIQUE REFERENCES analysis_results(id) ON DELETE CASCADE,
    total_pages INTEGER NOT NULL DEFAULT 0,
    pages_crawled INTEGER NOT NULL DEFAULT 0,
    total_issues INTEGER NOT NULL DEFAULT 0,
    critical_issues INTEGER NOT NULL DEFAULT 0,
    warning_issues INTEGER NOT NULL DEFAULT 0,
    info_issues INTEGER NOT NULL DEFAULT 0
);

-- Original analysis_issues table (aggregate)
CREATE TABLE analysis_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_id TEXT NOT NULL REFERENCES analysis_results(id) ON DELETE CASCADE,
    issue_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    UNIQUE (result_id, issue_type, severity)
);

-- Original settings table
CREATE TABLE settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    openai_api_key TEXT,
    anthropic_api_key TEXT,
    gemini_api_key TEXT,
    openai_enabled INTEGER NOT NULL DEFAULT 1,
    anthropic_enabled INTEGER NOT NULL DEFAULT 0,
    gemini_enabled INTEGER NOT NULL DEFAULT 0,
    system_prompt TEXT,
    user_prompt_template TEXT,
    include_page_content INTEGER NOT NULL DEFAULT 1,
    include_headings INTEGER NOT NULL DEFAULT 1,
    include_links INTEGER NOT NULL DEFAULT 1,
    include_images INTEGER NOT NULL DEFAULT 1,
    include_lighthouse INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Original analysis_ai_insights table
CREATE TABLE analysis_ai_insights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    result_id TEXT NOT NULL UNIQUE REFERENCES analysis_results(id) ON DELETE CASCADE,
    insight_text TEXT,
    recommendations TEXT,
    model_used TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Step 5: Recreate original indexes
CREATE INDEX idx_analysis_jobs_status ON analysis_jobs(status);
CREATE INDEX idx_analysis_jobs_created_at ON analysis_jobs(created_at);
CREATE INDEX idx_page_analysis_result_id ON page_analysis(result_id);
CREATE INDEX idx_page_analysis_url ON page_analysis(url);
CREATE INDEX idx_seo_issues_result_id ON seo_issues(result_id);
CREATE INDEX idx_seo_issues_page_id ON seo_issues(page_id);
CREATE INDEX idx_seo_issues_severity ON seo_issues(severity);
CREATE INDEX idx_page_edge_source ON page_edge(source_page_id);
CREATE INDEX idx_page_edge_target ON page_edge(target_page_id);

-- Note: Data migration back from new schema is not included
-- as it would require the new tables to still exist.
-- This down migration is primarily for schema rollback.
