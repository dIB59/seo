-- =============================================================================
-- Migration 0019: Fix foreign key references after schema redesign
-- =============================================================================
-- Some databases ended up with FK references to jobs_new/pages_new after the
-- rename in 0018. Rebuild tables to reference jobs/pages correctly.

PRAGMA foreign_keys = OFF;

-- Rename existing tables
ALTER TABLE pages RENAME TO pages_old;
ALTER TABLE issues RENAME TO issues_old;
ALTER TABLE links RENAME TO links_old;
ALTER TABLE page_lighthouse RENAME TO page_lighthouse_old;
ALTER TABLE page_headings RENAME TO page_headings_old;
ALTER TABLE page_images RENAME TO page_images_old;
ALTER TABLE ai_insights RENAME TO ai_insights_old;

-- Recreate tables with correct FK references
CREATE TABLE IF NOT EXISTS pages (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
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
    crawled_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (job_id, url)
);

CREATE TABLE IF NOT EXISTS issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    page_id TEXT REFERENCES pages(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('critical', 'warning', 'info')),
    message TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    source_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    target_page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    target_url TEXT NOT NULL,
    link_text TEXT,
    link_type TEXT NOT NULL DEFAULT 'internal' CHECK (link_type IN ('internal', 'external', 'resource')),
    is_followed INTEGER NOT NULL DEFAULT 1,
    status_code INTEGER,
    UNIQUE (source_page_id, target_url)
);

CREATE TABLE IF NOT EXISTS page_lighthouse (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL UNIQUE REFERENCES pages(id) ON DELETE CASCADE,
    performance_score REAL,
    accessibility_score REAL,
    best_practices_score REAL,
    seo_score REAL,
    first_contentful_paint_ms REAL,
    largest_contentful_paint_ms REAL,
    total_blocking_time_ms REAL,
    cumulative_layout_shift REAL,
    speed_index REAL,
    time_to_interactive_ms REAL,
    raw_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS page_headings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    level INTEGER NOT NULL CHECK (level >= 1 AND level <= 6),
    text TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS page_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    src TEXT NOT NULL,
    alt TEXT,
    width INTEGER,
    height INTEGER,
    loading TEXT,
    is_decorative INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS ai_insights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL UNIQUE REFERENCES jobs(id) ON DELETE CASCADE,
    summary TEXT,
    recommendations TEXT,
    raw_response TEXT,
    model TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Copy data
INSERT INTO pages SELECT * FROM pages_old;
INSERT INTO issues SELECT * FROM issues_old;
INSERT INTO links SELECT * FROM links_old;
INSERT INTO page_lighthouse SELECT * FROM page_lighthouse_old;
INSERT INTO page_headings SELECT * FROM page_headings_old;
INSERT INTO page_images SELECT * FROM page_images_old;
INSERT INTO ai_insights SELECT * FROM ai_insights_old;

-- Drop old tables
DROP TABLE pages_old;
DROP TABLE issues_old;
DROP TABLE links_old;
DROP TABLE page_lighthouse_old;
DROP TABLE page_headings_old;
DROP TABLE page_images_old;
DROP TABLE ai_insights_old;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_pages_job_id ON pages(job_id);
CREATE INDEX IF NOT EXISTS idx_pages_job_url ON pages(job_id, url);
CREATE INDEX IF NOT EXISTS idx_pages_job_depth ON pages(job_id, depth);

CREATE INDEX IF NOT EXISTS idx_issues_job_id ON issues(job_id);
CREATE INDEX IF NOT EXISTS idx_issues_job_severity ON issues(job_id, severity);
CREATE INDEX IF NOT EXISTS idx_issues_page_id ON issues(page_id);

CREATE INDEX IF NOT EXISTS idx_links_job_id ON links(job_id);
CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_page_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_page_id);

CREATE INDEX IF NOT EXISTS idx_page_headings_page_id ON page_headings(page_id);
CREATE INDEX IF NOT EXISTS idx_page_images_page_id ON page_images(page_id);

-- Recreate triggers for stats and updated_at
DROP TRIGGER IF EXISTS trg_update_job_stats_on_page_insert;
DROP TRIGGER IF EXISTS trg_update_job_stats_on_issue_insert;

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

PRAGMA foreign_keys = ON;
