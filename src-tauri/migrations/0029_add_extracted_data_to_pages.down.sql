-- =============================================================================
-- Migration 0029: Add extracted_data column to pages table
-- =============================================================================
-- Rollback: Remove the extracted_data column from pages table
-- =============================================================================

-- Note: SQLite doesn't support DROP COLUMN directly in older versions
-- We need to recreate the table
ALTER TABLE pages DROP COLUMN extracted_data;
