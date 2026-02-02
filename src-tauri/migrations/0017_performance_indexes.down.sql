-- Rollback performance indexes

DROP INDEX IF EXISTS ix_page_edge_from_page_covering;
DROP INDEX IF EXISTS ix_seo_issues_page_type;
DROP INDEX IF EXISTS ix_analysis_jobs_id_result;
DROP INDEX IF EXISTS ix_analysis_jobs_pending;
