-- =============================================================================
-- Migration 0030: Drop Extension Tables
-- =============================================================================
-- The extension system has been removed. These tables are dead code.
-- =============================================================================

DROP TABLE IF EXISTS page_href_tags;
DROP TABLE IF EXISTS page_keywords;
DROP TABLE IF EXISTS page_extensions;
DROP TABLE IF EXISTS audit_checks;
DROP TABLE IF EXISTS extractor_configs;
DROP TABLE IF EXISTS audit_rules;
