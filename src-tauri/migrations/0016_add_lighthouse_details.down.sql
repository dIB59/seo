-- Remove detailed Lighthouse columns
ALTER TABLE page_analysis DROP COLUMN lighthouse_seo_audits;
ALTER TABLE page_analysis DROP COLUMN lighthouse_performance_metrics;
