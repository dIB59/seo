-- =============================================================================
-- Migration 0028: Drop Extension Tables (Rollback)
-- =============================================================================

DROP TABLE IF EXISTS audit_checks;
DROP TABLE IF EXISTS page_href_tags;
DROP TABLE IF EXISTS page_keywords;
DROP TABLE IF EXISTS page_extensions;
DROP TABLE IF EXISTS extractor_configs;
DROP TABLE IF EXISTS audit_rules;
