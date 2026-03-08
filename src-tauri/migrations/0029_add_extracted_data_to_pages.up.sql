-- =============================================================================
-- Migration 0029: Add extracted_data column to pages table
-- =============================================================================
-- This migration adds a column to store extracted data from custom extractors
-- directly in the pages table as JSON.
-- =============================================================================

ALTER TABLE pages ADD COLUMN extracted_data TEXT NOT NULL DEFAULT '{}';
