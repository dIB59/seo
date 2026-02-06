-- Add up migration script here
-- =============================================================================
-- Migration 0020: Recreate views to reference `issues` table
-- =============================================================================
-- Ensure views created by 0018 reference the final `issues` table after prior
-- migrations that recreated the `issues` table. This is safe to run idempotently.
-- Created: 2026-02-04
-- =============================================================================

PRAGMA foreign_keys = OFF;

-- Drop views if they exist so we can recreate them cleanly
DROP VIEW IF EXISTS v_page_with_issues;
DROP VIEW IF EXISTS v_issues_by_severity;
DROP VIEW IF EXISTS v_job_summary;

PRAGMA foreign_keys = ON;

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
