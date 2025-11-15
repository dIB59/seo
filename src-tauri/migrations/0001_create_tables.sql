-- ==========================================
-- MIGRATION 001: Initial Schema
-- Created: 2025-11-15
-- Description: Create initial SEO analysis database schema
-- ==========================================

PRAGMA foreign_keys = ON;

-- ==========================================
-- 1. ANALYSIS SETTINGS (Configuration Templates)
-- ==========================================
CREATE TABLE IF NOT EXISTS analysis_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    max_pages INTEGER NOT NULL,
    include_external_links INTEGER NOT NULL,
    check_images INTEGER NOT NULL,
    mobile_analysis INTEGER NOT NULL,
    lighthouse_analysis INTEGER NOT NULL,
    delay_between_requests INTEGER NOT NULL,
    created_at DATETIME DEFAULT (datetime('now'))
);

-- ==========================================
-- 2. ANALYSIS JOBS (Job Queue)
-- ==========================================
CREATE TABLE IF NOT EXISTS analysis_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    settings_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'processing', 'completed', 'failed')),
    result_id TEXT,
    FOREIGN KEY (settings_id) REFERENCES analysis_settings(id) ON DELETE CASCADE,
    FOREIGN KEY (result_id) REFERENCES analysis_results(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS ix_analysis_jobs_status ON analysis_jobs(status);
CREATE INDEX IF NOT EXISTS ix_analysis_jobs_created_at ON analysis_jobs(created_at);
CREATE INDEX IF NOT EXISTS ix_analysis_jobs_result_id ON analysis_jobs(result_id);

-- ==========================================
-- 3. ANALYSIS RESULTS (Main Results Entity)
-- ==========================================
CREATE TABLE IF NOT EXISTS analysis_results (
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
    created_at DATETIME DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS ix_analysis_results_status ON analysis_results(status);
CREATE INDEX IF NOT EXISTS ix_analysis_results_url ON analysis_results(url);
CREATE INDEX IF NOT EXISTS ix_analysis_results_created_at ON analysis_results(created_at);

-- ==========================================
-- 4. ANALYSIS ISSUES (Issue Counts Summary)
-- ==========================================
CREATE TABLE IF NOT EXISTS analysis_issues (
    analysis_id TEXT PRIMARY KEY,
    critical INTEGER NOT NULL DEFAULT 0,
    warnings INTEGER NOT NULL DEFAULT 0,
    suggestions INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (analysis_id) REFERENCES analysis_results(id) ON DELETE CASCADE
);

-- ==========================================
-- 5. ANALYSIS SUMMARY (Aggregated Statistics)
-- ==========================================
CREATE TABLE IF NOT EXISTS analysis_summary (
    analysis_id TEXT PRIMARY KEY,
    avg_load_time REAL NOT NULL DEFAULT 0,
    total_words INTEGER NOT NULL DEFAULT 0,
    pages_with_issues INTEGER NOT NULL DEFAULT 0,
    seo_score INTEGER NOT NULL DEFAULT 0 CHECK (seo_score >= 0 AND seo_score <= 100),
    mobile_friendly_pages INTEGER NOT NULL DEFAULT 0,
    pages_with_meta_description INTEGER NOT NULL DEFAULT 0,
    pages_with_title_issues INTEGER NOT NULL DEFAULT 0,
    duplicate_titles INTEGER NOT NULL DEFAULT 0,
    duplicate_meta_descriptions INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (analysis_id) REFERENCES analysis_results(id) ON DELETE CASCADE
);

-- ==========================================
-- 6. PAGE ANALYSIS (Individual Page Results)
-- ==========================================
CREATE TABLE IF NOT EXISTS page_analysis (
    id TEXT PRIMARY KEY,
    analysis_id TEXT NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    meta_description TEXT,
    meta_keywords TEXT,
    canonical_url TEXT,
    h1_count INTEGER NOT NULL DEFAULT 0,
    h2_count INTEGER NOT NULL DEFAULT 0,
    h3_count INTEGER NOT NULL DEFAULT 0,
    word_count INTEGER NOT NULL DEFAULT 0,
    image_count INTEGER NOT NULL DEFAULT 0,
    images_without_alt INTEGER NOT NULL DEFAULT 0,
    internal_links INTEGER NOT NULL DEFAULT 0,
    external_links INTEGER NOT NULL DEFAULT 0,
    load_time REAL NOT NULL DEFAULT 0,
    status_code INTEGER,
    content_size INTEGER NOT NULL DEFAULT 0,
    mobile_friendly BOOLEAN NOT NULL DEFAULT 0,
    has_structured_data BOOLEAN NOT NULL DEFAULT 0,
    lighthouse_performance REAL,
    lighthouse_accessibility REAL,
    lighthouse_best_practices REAL,
    lighthouse_seo REAL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (analysis_id) REFERENCES analysis_results(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_page_analysis_analysis_url ON page_analysis(analysis_id, url);
CREATE INDEX IF NOT EXISTS ix_page_analysis_analysis_id ON page_analysis(analysis_id);
CREATE INDEX IF NOT EXISTS ix_page_analysis_created_at ON page_analysis(created_at);
CREATE INDEX IF NOT EXISTS ix_page_analysis_status_code ON page_analysis(status_code);

-- ==========================================
-- 7. SEO ISSUES (Detailed Issues per Page)
-- ==========================================
CREATE TABLE IF NOT EXISTS seo_issues (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('critical','warning','suggestion')),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    page_url TEXT NOT NULL,
    element TEXT,
    line_number INTEGER,
    recommendation TEXT NOT NULL,
    created_at DATETIME DEFAULT (datetime('now')),
    FOREIGN KEY (page_id) REFERENCES page_analysis(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_seo_issues_page_id ON seo_issues(page_id);
CREATE INDEX IF NOT EXISTS ix_seo_issues_type ON seo_issues(type);
CREATE INDEX IF NOT EXISTS ix_seo_issues_created_at ON seo_issues(created_at);

-- ==========================================
-- VIEWS FOR COMMON QUERIES
-- ==========================================

-- Complete analysis overview
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

-- Page details with issue counts
CREATE VIEW IF NOT EXISTS v_page_details AS
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

-- Analysis job tracking
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

-- ==========================================
-- SEED DATA (Optional)
-- ==========================================

-- Insert default analysis settings
INSERT OR IGNORE INTO analysis_settings (
    id, 
    max_pages, 
    include_external_links, 
    check_images, 
    mobile_analysis, 
    lighthouse_analysis, 
    delay_between_requests
) VALUES (
    1,
    100,
    1,
    1,
    1,
    1,
    1000
);
