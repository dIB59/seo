-- Create page_queue table for tracking pages to analyze
-- This enables resumability, concurrent page analysis, and individual page status tracking

CREATE TABLE IF NOT EXISTS page_queue (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    url TEXT NOT NULL,
    depth INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    -- status: pending, processing, completed, failed
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

-- Index for efficient job+status queries (used by workers to claim pages)
CREATE INDEX IF NOT EXISTS idx_page_queue_job_status ON page_queue(job_id, status);

-- Index for finding pending pages across all jobs
CREATE INDEX IF NOT EXISTS idx_page_queue_status ON page_queue(status);

-- Index for cleanup and job-level operations
CREATE INDEX IF NOT EXISTS idx_page_queue_job_id ON page_queue(job_id);
