-- Add columns for detailed Lighthouse audit data
ALTER TABLE page_analysis ADD COLUMN lighthouse_seo_audits TEXT;
ALTER TABLE page_analysis ADD COLUMN lighthouse_performance_metrics TEXT;
