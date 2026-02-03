-- =============================================================================
-- Migration 0018: Complete Schema Redesign
-- =============================================================================
-- Goals:
-- 1. Merge analysis_jobs + analysis_results + analysis_settings into single 'jobs' table
-- 2. Add direct job_id FK to pages, issues, links (eliminate expensive JOINs)
-- 3. Normalize JSON blobs (headings, images) into proper tables
-- 4. Remove redundant aggregate tables (analysis_summary, analysis_issues)
-- 5. Add proper indexes and constraints
-- =============================================================================

-- Disable foreign key checks during migration to allow data migration
PRAGMA foreign_keys = OFF;

-- Step 1: Create new normalized schema
-- -----------------------------------------------------------------------------

-- Jobs table: Consolidates job metadata, settings, and summary
CREATE TABLE IF NOT EXISTS jobs_new (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    
    -- Settings (from analysis_settings)
    max_pages INTEGER NOT NULL DEFAULT 100,
    max_depth INTEGER NOT NULL DEFAULT 3,
    respect_robots_txt INTEGER NOT NULL DEFAULT 1,
    include_subdomains INTEGER NOT NULL DEFAULT 0,
    rate_limit_ms INTEGER NOT NULL DEFAULT 1000,
    user_agent TEXT,
    
    -- Summary stats (computed, denormalized for fast access)
    total_pages INTEGER NOT NULL DEFAULT 0,
    pages_crawled INTEGER NOT NULL DEFAULT 0,
    total_issues INTEGER NOT NULL DEFAULT 0,
    critical_issues INTEGER NOT NULL DEFAULT 0,
    warning_issues INTEGER NOT NULL DEFAULT 0,
    info_issues INTEGER NOT NULL DEFAULT 0,
    
    -- Progress tracking
    progress REAL NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 100.0),
    current_stage TEXT,
    error_message TEXT
);

-- Pages table: One row per crawled page with direct job reference
CREATE TABLE IF NOT EXISTS pages_new (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL REFERENCES jobs_new(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    depth INTEGER NOT NULL DEFAULT 0,
    status_code INTEGER,
    content_type TEXT,
    
    -- Core SEO fields
    title TEXT,
    meta_description TEXT,
    canonical_url TEXT,
    robots_meta TEXT,
    
    -- Content metrics
    word_count INTEGER,
    load_time_ms INTEGER,
    response_size_bytes INTEGER,
    
    -- Timestamps
    crawled_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- Unique constraint: one URL per job
    UNIQUE (job_id, url)
);

-- Issues table: Direct job_id FK for fast queries
CREATE TABLE IF NOT EXISTS issues_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL REFERENCES jobs_new(id) ON DELETE CASCADE,
    page_id TEXT REFERENCES pages_new(id) ON DELETE CASCADE,
    
    type TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('critical', 'warning', 'info')),
    message TEXT NOT NULL,
    details TEXT,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Links table: Direct job_id FK, normalized from page edges
CREATE TABLE IF NOT EXISTS links_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL REFERENCES jobs_new(id) ON DELETE CASCADE,
    source_page_id TEXT NOT NULL REFERENCES pages_new(id) ON DELETE CASCADE,
    target_page_id TEXT REFERENCES pages_new(id) ON DELETE SET NULL,
    
    target_url TEXT NOT NULL,
    link_text TEXT,
    link_type TEXT NOT NULL DEFAULT 'internal' CHECK (link_type IN ('internal', 'external', 'resource')),
    is_followed INTEGER NOT NULL DEFAULT 1,
    status_code INTEGER,
    
    -- Prevent duplicate links
    UNIQUE (source_page_id, target_url)
);

-- Lighthouse data: Separate table for detailed performance metrics
CREATE TABLE IF NOT EXISTS page_lighthouse (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL UNIQUE REFERENCES pages_new(id) ON DELETE CASCADE,
    
    -- Core Web Vitals
    performance_score REAL,
    accessibility_score REAL,
    best_practices_score REAL,
    seo_score REAL,
    
    -- Performance metrics
    first_contentful_paint_ms REAL,
    largest_contentful_paint_ms REAL,
    total_blocking_time_ms REAL,
    cumulative_layout_shift REAL,
    speed_index REAL,
    time_to_interactive_ms REAL,
    
    -- Raw data for detailed analysis
    raw_json TEXT,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Headings: Normalized from JSON blob
CREATE TABLE IF NOT EXISTS page_headings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages_new(id) ON DELETE CASCADE,
    
    level INTEGER NOT NULL CHECK (level >= 1 AND level <= 6),
    text TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0
);

-- Images: Normalized from JSON blob
CREATE TABLE IF NOT EXISTS page_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages_new(id) ON DELETE CASCADE,
    
    src TEXT NOT NULL,
    alt TEXT,
    width INTEGER,
    height INTEGER,
    loading TEXT,
    is_decorative INTEGER NOT NULL DEFAULT 0
);

-- AI Insights: Per-job AI analysis
CREATE TABLE IF NOT EXISTS ai_insights_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL UNIQUE REFERENCES jobs_new(id) ON DELETE CASCADE,
    
    summary TEXT,
    recommendations TEXT,
    raw_response TEXT,
    model TEXT,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Settings: Global app settings (unchanged but recreated for consistency)
CREATE TABLE IF NOT EXISTS settings_new (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    
    -- AI Settings
    openai_api_key TEXT,
    anthropic_api_key TEXT,
    google_api_key TEXT,
    default_ai_provider TEXT DEFAULT 'openai',
    
    -- Crawl defaults
    default_max_pages INTEGER DEFAULT 100,
    default_max_depth INTEGER DEFAULT 3,
    default_rate_limit_ms INTEGER DEFAULT 1000,
    
    -- UI preferences
    theme TEXT DEFAULT 'system',
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Step 2: Create indexes for common query patterns
-- -----------------------------------------------------------------------------

-- Jobs indexes
CREATE INDEX IF NOT EXISTS idx_jobs_new_status ON jobs_new(status);
CREATE INDEX IF NOT EXISTS idx_jobs_new_created_at ON jobs_new(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_jobs_new_url ON jobs_new(url);

-- Pages indexes (covering indexes for common queries)
CREATE INDEX IF NOT EXISTS idx_pages_new_job_id ON pages_new(job_id);
CREATE INDEX IF NOT EXISTS idx_pages_new_job_url ON pages_new(job_id, url);
CREATE INDEX IF NOT EXISTS idx_pages_new_job_depth ON pages_new(job_id, depth);

-- Issues indexes (optimized for dashboard queries)
CREATE INDEX IF NOT EXISTS idx_issues_new_job_id ON issues_new(job_id);
CREATE INDEX IF NOT EXISTS idx_issues_new_job_severity ON issues_new(job_id, severity);
CREATE INDEX IF NOT EXISTS idx_issues_new_page_id ON issues_new(page_id);

-- Links indexes
CREATE INDEX IF NOT EXISTS idx_links_new_job_id ON links_new(job_id);
CREATE INDEX IF NOT EXISTS idx_links_new_source ON links_new(source_page_id);
CREATE INDEX IF NOT EXISTS idx_links_new_target ON links_new(target_page_id);

-- Headings/Images indexes
CREATE INDEX IF NOT EXISTS idx_page_headings_page_id ON page_headings(page_id);
CREATE INDEX IF NOT EXISTS idx_page_images_page_id ON page_images(page_id);

-- Step 3: Migrate data from old schema to new schema
-- -----------------------------------------------------------------------------

-- Migrate jobs (merge analysis_jobs + analysis_results + analysis_settings)
-- Map old status values to new ones and use analysis_results for progress info
INSERT INTO jobs_new (
    id, url, status, created_at, updated_at, completed_at,
    max_pages, max_depth, respect_robots_txt, include_subdomains, rate_limit_ms, user_agent,
    total_pages, pages_crawled, total_issues, critical_issues, warning_issues, info_issues,
    progress, current_stage, error_message
)
SELECT 
    CAST(j.id AS TEXT),
    j.url,
    CASE j.status 
        WHEN 'queued' THEN 'pending'
        WHEN 'processing' THEN 'running'
        WHEN 'discovering' THEN 'running'
        WHEN 'completed' THEN 'completed'
        WHEN 'failed' THEN 'failed'
        ELSE 'pending'
    END,
    j.created_at,
    COALESCE(j.updated_at, j.created_at),
    ar.completed_at,
    COALESCE(s.max_pages, 100),
    3, -- max_depth not in old schema, use default
    1, -- respect_robots_txt not in old schema, use default
    0, -- include_subdomains not in old schema, use default
    COALESCE(s.delay_between_requests, 1000),
    NULL, -- user_agent not in old schema
    COALESCE(ar.total_pages, 0),
    COALESCE(ar.analyzed_pages, 0),
    COALESCE((SELECT COUNT(*) FROM seo_issues si 
              JOIN page_analysis pa ON si.page_id = pa.id 
              WHERE pa.analysis_id = j.result_id), 0),
    COALESCE((SELECT COUNT(*) FROM seo_issues si 
              JOIN page_analysis pa ON si.page_id = pa.id 
              WHERE pa.analysis_id = j.result_id AND si.type = 'critical'), 0),
    COALESCE((SELECT COUNT(*) FROM seo_issues si 
              JOIN page_analysis pa ON si.page_id = pa.id 
              WHERE pa.analysis_id = j.result_id AND si.type = 'warning'), 0),
    COALESCE((SELECT COUNT(*) FROM seo_issues si 
              JOIN page_analysis pa ON si.page_id = pa.id 
              WHERE pa.analysis_id = j.result_id AND si.type = 'suggestion'), 0),
    COALESCE(ar.progress, 0.0),
    NULL, -- current_stage not in old schema
    NULL  -- error_message not in old schema
FROM analysis_jobs j
LEFT JOIN analysis_settings s ON s.id = j.settings_id
LEFT JOIN analysis_results ar ON ar.id = j.result_id;

-- Migrate pages (from page_analysis)
-- Note: depth, content_type, robots_meta not in old schema
INSERT INTO pages_new (
    id, job_id, url, depth, status_code, content_type,
    title, meta_description, canonical_url, robots_meta,
    word_count, load_time_ms, response_size_bytes, crawled_at
)
SELECT 
    pa.id,
    CAST(j.id AS TEXT) as job_id,
    pa.url,
    0, -- depth not in old schema
    pa.status_code,
    NULL, -- content_type not in old schema
    pa.title,
    pa.meta_description,
    pa.canonical_url,
    NULL, -- robots_meta not in old schema
    pa.word_count,
    CAST(pa.load_time * 1000 AS INTEGER), -- convert seconds to ms
    pa.content_size,
    COALESCE(pa.created_at, datetime('now'))
FROM page_analysis pa
JOIN analysis_jobs j ON j.result_id = pa.analysis_id
WHERE j.result_id IS NOT NULL;

-- Migrate issues (from seo_issues, adding job_id)
-- Map old 'type' column (critical/warning/suggestion) to severity
INSERT INTO issues_new (job_id, page_id, type, severity, message, details, created_at)
SELECT 
    CAST(j.id AS TEXT) as job_id,
    si.page_id,
    si.title, -- use title as issue type
    CASE si.type 
        WHEN 'critical' THEN 'critical'
        WHEN 'warning' THEN 'warning'
        WHEN 'suggestion' THEN 'info'
        ELSE 'info'
    END,
    si.description, -- use description as message
    si.recommendation, -- use recommendation as details
    COALESCE(si.created_at, datetime('now'))
FROM seo_issues si
JOIN page_analysis pa ON pa.id = si.page_id
JOIN analysis_jobs j ON j.result_id = pa.analysis_id
WHERE j.result_id IS NOT NULL;

-- Migrate links (from page_edge, adding job_id)
-- Old schema uses from_page_id/to_url instead of source_page_id/target_page_id
-- Use OR IGNORE to skip duplicates (old schema may have duplicate links)
INSERT OR IGNORE INTO links_new (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    CAST(j.id AS TEXT) as job_id,
    pe.from_page_id,
    NULL, -- target_page_id needs to be resolved
    pe.to_url,
    NULL, -- link_text not in old schema
    'internal', -- link_type not in old schema, assume internal
    1, -- is_followed not in old schema, assume true
    pe.status_code
FROM page_edge pe
JOIN page_analysis source_pa ON source_pa.id = pe.from_page_id
JOIN analysis_jobs j ON j.result_id = source_pa.analysis_id
WHERE j.result_id IS NOT NULL;

-- Migrate lighthouse data (scores already in page_analysis)
-- Note: Old schema doesn't have detailed lighthouse metrics (fcp, lcp, etc.)
INSERT INTO page_lighthouse (
    page_id, performance_score, accessibility_score, best_practices_score, seo_score,
    first_contentful_paint_ms, largest_contentful_paint_ms, total_blocking_time_ms,
    cumulative_layout_shift, speed_index, time_to_interactive_ms, raw_json
)
SELECT 
    pa.id,
    pa.lighthouse_performance,
    pa.lighthouse_accessibility,
    pa.lighthouse_best_practices,
    pa.lighthouse_seo,
    NULL, -- fcp not in old schema
    NULL, -- lcp not in old schema
    NULL, -- tbt not in old schema
    NULL, -- cls not in old schema
    NULL, -- si not in old schema
    NULL, -- tti not in old schema
    pa.lighthouse_performance_metrics -- raw json if available
FROM page_analysis pa
WHERE pa.lighthouse_performance IS NOT NULL;

-- Migrate AI insights
-- Old schema stores as key-value in settings or in analysis_ai_insights
INSERT INTO ai_insights_new (job_id, summary, recommendations, model, created_at, updated_at)
SELECT 
    CAST(j.id AS TEXT) as job_id,
    ai.insights,
    NULL, -- recommendations not separate in old schema
    NULL, -- model not in old schema
    ai.created_at,
    ai.created_at
FROM analysis_ai_insights ai
JOIN analysis_jobs j ON j.result_id = ai.analysis_id;

-- Migrate global settings (old schema uses key-value pairs)
-- Insert default settings since old schema is different
INSERT OR REPLACE INTO settings_new (
    id, openai_api_key, anthropic_api_key, google_api_key, default_ai_provider,
    default_max_pages, default_max_depth, default_rate_limit_ms, theme
)
VALUES (
    1,
    (SELECT value FROM settings WHERE key = 'openai_api_key'),
    (SELECT value FROM settings WHERE key = 'anthropic_api_key'),
    (SELECT value FROM settings WHERE key = 'google_api_key'),
    COALESCE((SELECT value FROM settings WHERE key = 'default_ai_provider'), 'openai'),
    100,
    3,
    1000,
    'system'
);

-- Step 4: Drop old tables and rename new ones
-- -----------------------------------------------------------------------------

-- Drop views that depend on old tables first
DROP VIEW IF EXISTS v_job_progress;
DROP VIEW IF EXISTS v_crawl_stats;
DROP VIEW IF EXISTS v_analysis_overview;
DROP VIEW IF EXISTS v_analysis_jobs;
DROP VIEW IF EXISTS v_page_details;

-- Drop old tables
DROP TABLE IF EXISTS analysis_ai_insights;
DROP TABLE IF EXISTS analysis_issues;
DROP TABLE IF EXISTS analysis_summary;
DROP TABLE IF EXISTS page_edge;
DROP TABLE IF EXISTS seo_issues;
DROP TABLE IF EXISTS page_analysis;
DROP TABLE IF EXISTS analysis_settings;
DROP TABLE IF EXISTS analysis_results;
DROP TABLE IF EXISTS analysis_jobs;
DROP TABLE IF EXISTS settings;

-- Rename new tables to final names
ALTER TABLE jobs_new RENAME TO jobs;
ALTER TABLE pages_new RENAME TO pages;
ALTER TABLE issues_new RENAME TO issues;
ALTER TABLE links_new RENAME TO links;
ALTER TABLE ai_insights_new RENAME TO ai_insights;
ALTER TABLE settings_new RENAME TO settings;

-- Step 5: Create views for backward compatibility and common queries
-- -----------------------------------------------------------------------------

-- View: Job summary with all stats
CREATE VIEW IF NOT EXISTS v_job_summary AS
SELECT 
    j.*,
    (SELECT COUNT(*) FROM pages p WHERE p.job_id = j.id) as actual_page_count,
    (SELECT COUNT(*) FROM issues i WHERE i.job_id = j.id) as actual_issue_count,
    (SELECT AVG(pl.performance_score) FROM page_lighthouse pl 
     JOIN pages p ON p.id = pl.page_id WHERE p.job_id = j.id) as avg_performance_score
FROM jobs j;

-- View: Page with issue counts
CREATE VIEW IF NOT EXISTS v_page_with_issues AS
SELECT 
    p.*,
    (SELECT COUNT(*) FROM issues i WHERE i.page_id = p.id AND i.severity = 'critical') as critical_count,
    (SELECT COUNT(*) FROM issues i WHERE i.page_id = p.id AND i.severity = 'warning') as warning_count,
    (SELECT COUNT(*) FROM issues i WHERE i.page_id = p.id AND i.severity = 'info') as info_count,
    pl.performance_score,
    pl.seo_score
FROM pages p
LEFT JOIN page_lighthouse pl ON pl.page_id = p.id;

-- View: Issues by severity for dashboard
CREATE VIEW IF NOT EXISTS v_issues_by_severity AS
SELECT 
    job_id,
    severity,
    type,
    COUNT(*) as count,
    GROUP_CONCAT(DISTINCT message) as messages
FROM issues
GROUP BY job_id, severity, type;

-- Step 6: Create triggers for updated_at timestamps
-- -----------------------------------------------------------------------------

CREATE TRIGGER IF NOT EXISTS trg_jobs_updated_at
AFTER UPDATE ON jobs
BEGIN
    UPDATE jobs SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_settings_updated_at
AFTER UPDATE ON settings
BEGIN
    UPDATE settings SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_ai_insights_updated_at
AFTER UPDATE ON ai_insights
BEGIN
    UPDATE ai_insights SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Step 7: Update job stats after page/issue changes
-- -----------------------------------------------------------------------------

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

-- Re-enable foreign key checks
PRAGMA foreign_keys = ON;
