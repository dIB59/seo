-- Performance optimization indexes
-- Created: 2026-02-02

-- Covering index for edge retrieval join (most expensive query)
-- This allows the JOIN in get_result_by_job_id to be resolved entirely from the index
CREATE INDEX IF NOT EXISTS ix_page_edge_from_page_covering 
ON page_edge(from_page_id, to_url, status_code);

-- Composite index for seo_issues join with page_analysis
-- Optimizes the issues query in get_result_by_job_id
CREATE INDEX IF NOT EXISTS ix_seo_issues_page_type 
ON seo_issues(page_id, type);

-- Index for job progress polling (happens every second during analysis)
CREATE INDEX IF NOT EXISTS ix_analysis_jobs_id_result 
ON analysis_jobs(id, result_id);

-- Partial index for pending jobs (only indexes non-terminal states)
-- Dramatically speeds up get_pending_jobs which is called frequently
CREATE INDEX IF NOT EXISTS ix_analysis_jobs_pending 
ON analysis_jobs(created_at) 
WHERE status IN ('queued', 'processing', 'discovering');
