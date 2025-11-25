-- ==========================================
-- MIGRATION 001 DOWN: Revert Initial Schema
-- Removes all tables, views, and indexes created by the initial migration
-- ==========================================

PRAGMA foreign_keys = OFF;

-- Drop all views first (they depend on tables)
DROP VIEW IF EXISTS v_analysis_jobs;
DROP VIEW IF EXISTS v_page_details;
DROP VIEW IF EXISTS v_analysis_overview;

-- Drop all tables in reverse dependency order
DROP TABLE IF EXISTS seo_issues;           -- References page_analysis
DROP TABLE IF EXISTS page_analysis;        -- References analysis_results  
DROP TABLE IF EXISTS analysis_summary;     -- References analysis_results
DROP TABLE IF EXISTS analysis_issues;      -- References analysis_results
DROP TABLE IF EXISTS analysis_jobs;        -- References analysis_settings, analysis_results
DROP TABLE IF EXISTS analysis_results;     -- No dependencies
DROP TABLE IF EXISTS analysis_settings;    -- No dependencies

PRAGMA foreign_keys = ON;