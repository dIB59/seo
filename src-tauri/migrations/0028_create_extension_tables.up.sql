-- =============================================================================
-- Migration 0028: Create Extension Tables
-- =============================================================================
-- This migration creates the tables needed for the extension system:
-- - audit_rules: Custom issue rules loaded from database
-- - extractor_configs: Custom data extractors
-- - page_extensions: Flexible storage for extracted data
-- - page_keywords: Keywords extracted from pages
-- - page_href_tags: Link tags from head section
-- - audit_checks: Custom audit checks
-- =============================================================================

-- Audit rules table
-- Stores custom issue rules that can be added without code changes
CREATE TABLE IF NOT EXISTS audit_rules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'seo',           -- seo, performance, accessibility
    severity TEXT NOT NULL DEFAULT 'warning',       -- critical, warning, info
    description TEXT,
    
    -- Rule definition
    rule_type TEXT NOT NULL,                        -- presence, threshold, length, regex, equals
    target_field TEXT NOT NULL,                     -- page field to check
    condition TEXT,                                 -- JSON condition definition
    threshold_value TEXT,                           -- JSON for threshold rules: {"min": 0, "max": 100}
    regex_pattern TEXT,                             -- For regex rules
    
    -- Metadata
    recommendation TEXT,                            -- How to fix
    learn_more_url TEXT,                            -- Link to documentation
    
    -- Control
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(name)
);

-- Extractor configurations table
-- Stores custom data extractors that can be added without code changes
CREATE TABLE IF NOT EXISTS extractor_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    
    -- Extraction definition
    extractor_type TEXT NOT NULL DEFAULT 'css_selector',  -- css_selector, xpath, regex, json_path
    selector TEXT NOT NULL,                         -- CSS selector, XPath, or regex pattern
    attribute TEXT,                                 -- Attribute to extract (null for text content)
    
    -- Processing
    post_process TEXT,                              -- JSON array of post-processing steps
    
    -- Storage
    storage_type TEXT NOT NULL DEFAULT 'json',      -- column, json, separate_table
    target_column TEXT,                             -- Column name if storage_type is column
    target_table TEXT,                              -- Table name if separate_table
    
    -- Control
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    CHECK (storage_type IN ('column', 'json', 'separate_table'))
);

-- Extended page data table
-- Flexible storage for data extracted by custom extractors
CREATE TABLE IF NOT EXISTS page_extensions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    extractor_id TEXT NOT NULL REFERENCES extractor_configs(id) ON DELETE CASCADE,
    
    -- Flexible data storage
    value_text TEXT,
    value_number REAL,
    value_json TEXT,                                -- JSON for complex data
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(page_id, extractor_id)
);

-- Keywords extracted from pages
CREATE TABLE IF NOT EXISTS page_keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    keyword TEXT NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    density REAL,                                   -- Percentage of total words
    is_meta_keyword INTEGER NOT NULL DEFAULT 0,     -- Whether it appears in meta keywords
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(page_id, keyword)
);

-- Href tags extracted from head section
CREATE TABLE IF NOT EXISTS page_href_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    rel TEXT NOT NULL,                              -- stylesheet, icon, canonical, etc.
    href TEXT NOT NULL,
    type TEXT,                                      -- MIME type if applicable
    sizes TEXT,                                     -- For icons
    media TEXT,                                     -- Media query
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(page_id, rel, href)
);

-- Audit checks table
-- Stores custom audit checks that contribute to SEO scores
CREATE TABLE IF NOT EXISTS audit_checks (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,                       -- Machine-readable key
    label TEXT NOT NULL,                            -- Human-readable label
    category TEXT NOT NULL DEFAULT 'seo',           -- seo, performance, accessibility
    
    -- Check definition
    check_type TEXT NOT NULL DEFAULT 'selector_count',  -- selector_count, field_check, custom
    selector TEXT,                                  -- CSS selector for selector_count
    field TEXT,                                     -- Page field for field_check
    condition TEXT NOT NULL DEFAULT '{}',           -- JSON condition
    
    -- Scoring
    weight REAL NOT NULL DEFAULT 1.0,               -- Weight in overall score
    pass_score REAL NOT NULL DEFAULT 1.0,
    fail_score REAL NOT NULL DEFAULT 0.0,
    
    -- Messages
    pass_message TEXT,
    fail_message TEXT,
    
    -- Control
    is_enabled INTEGER NOT NULL DEFAULT 1,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    CHECK (weight >= 0.0 AND weight <= 2.0)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_audit_rules_category ON audit_rules(category);
CREATE INDEX IF NOT EXISTS idx_audit_rules_enabled ON audit_rules(is_enabled);
CREATE INDEX IF NOT EXISTS idx_extractor_configs_enabled ON extractor_configs(is_enabled);
CREATE INDEX IF NOT EXISTS idx_page_extensions_page_id ON page_extensions(page_id);
CREATE INDEX IF NOT EXISTS idx_page_keywords_page_id ON page_keywords(page_id);
CREATE INDEX IF NOT EXISTS idx_page_keywords_keyword ON page_keywords(keyword);
CREATE INDEX IF NOT EXISTS idx_page_href_tags_page_id ON page_href_tags(page_id);
CREATE INDEX IF NOT EXISTS idx_audit_checks_category ON audit_checks(category);
CREATE INDEX IF NOT EXISTS idx_audit_checks_enabled ON audit_checks(is_enabled);

-- =============================================================================
-- Seed built-in rules
-- =============================================================================

-- These rules replicate the existing hardcoded behavior
-- They are marked as is_builtin = 1 so they can be differentiated from custom rules

INSERT INTO audit_rules (id, name, category, severity, description, rule_type, target_field, recommendation, is_enabled, is_builtin) VALUES
('missing-title', 'Missing Title', 'seo', 'critical', 'Page has no title tag', 'presence', 'title', 'Add a descriptive title tag', 1, 1),
('missing-meta-description', 'Missing Meta Description', 'seo', 'warning', 'Page has no meta description', 'presence', 'meta_description', 'Add a meta description', 1, 1);

INSERT INTO audit_rules (id, name, category, severity, description, rule_type, target_field, threshold_value, recommendation, is_enabled, is_builtin) VALUES
('title-length', 'Title Length', 'seo', 'warning', 'Title length should be between 30-60 characters', 'length', 'title', '{"min": 30, "max": 60}', 'Keep title between 30-60 characters', 1, 1),
('meta-description-length', 'Meta Description Length', 'seo', 'warning', 'Meta description length should be between 70-160 characters', 'length', 'meta_description', '{"min": 70, "max": 160}', 'Keep meta description between 70-160 characters', 1, 1),
('low-word-count', 'Low Word Count', 'seo', 'info', 'Page has low word count', 'threshold', 'word_count', '{"min": 300}', 'Consider adding more content (300+ words)', 1, 1),
('slow-load-time', 'Slow Page Load', 'performance', 'warning', 'Page load time exceeds 3 seconds', 'threshold', 'load_time_ms', '{"max": 3000}', 'Optimize page load time to under 3 seconds', 1, 1);

-- HTTP status code rule (special rule type)
INSERT INTO audit_rules (id, name, category, severity, description, rule_type, target_field, threshold_value, recommendation, is_enabled, is_builtin) VALUES
('http-error', 'HTTP Error', 'seo', 'critical', 'Page returned an HTTP error status code (4xx or 5xx)', 'status_code', 'status_code', '{"error_codes": [400, 401, 403, 404, 500, 502, 503, 504]}', 'Fix the HTTP error or remove the page', 1, 1);

-- Seed built-in extractors
INSERT INTO extractor_configs (id, name, display_name, description, extractor_type, selector, attribute, storage_type, is_enabled, is_builtin) VALUES
('open-graph', 'open_graph', 'Open Graph Tags', 'Extracts Open Graph meta tags from the page', 'css_selector', 'meta[property^="og:"]', 'content', 'json', 1, 1),
('twitter-card', 'twitter_card', 'Twitter Card Tags', 'Extracts Twitter Card meta tags from the page', 'css_selector', 'meta[name^="twitter:"]', 'content', 'json', 1, 1),
('href-tags', 'href_tags', 'Href Tags', 'Extracts link tags from the head section', 'css_selector', 'head link[href]', NULL, 'json', 1, 1),
('keywords', 'keywords', 'Keywords', 'Extracts keywords from page content based on frequency', 'css_selector', 'body', NULL, 'json', 1, 1),
('structured-data', 'structured_data', 'Structured Data', 'Extracts JSON-LD structured data from the page', 'css_selector', 'script[type="application/ld+json"]', NULL, 'json', 1, 1);

-- Seed built-in audit checks
INSERT INTO audit_checks (id, key, label, category, check_type, selector, weight, pass_message, fail_message, is_enabled, is_builtin) VALUES
('document-title', 'document_title', 'Document Title', 'seo', 'field_check', NULL, 1.0, 'Title length is good', 'Missing or invalid document title', 1, 1),
('meta-description', 'meta_description', 'Meta Description', 'seo', 'field_check', NULL, 1.0, 'Description length is good', 'Missing or invalid meta description', 1, 1),
('viewport', 'viewport', 'Viewport Meta Tag', 'seo', 'selector_count', 'meta[name="viewport"]', 1.0, 'Viewport is properly configured', 'Missing viewport meta tag', 1, 1),
('canonical', 'canonical', 'Canonical URL', 'seo', 'selector_count', 'link[rel="canonical"]', 0.8, 'Canonical URL is set', 'Missing canonical URL', 1, 1),
('hreflang', 'hreflang', 'Hreflang Tags', 'seo', 'selector_count', 'link[rel="alternate"][hreflang]', 0.5, 'Hreflang tags found', 'No hreflang tags (optional for single-language sites)', 1, 1),
('crawlable-anchors', 'crawlable_anchors', 'Crawlable Anchors', 'seo', 'selector_count', 'a[href]', 0.7, 'All links are crawlable', 'Some links are not crawlable', 1, 1),
('link-text', 'link_text', 'Descriptive Link Text', 'seo', 'field_check', NULL, 0.6, 'All links have descriptive text', 'Some links have generic/empty text', 1, 1),
('image-alt', 'image_alt', 'Image Alt Attributes', 'seo', 'selector_count', 'img', 0.8, 'All images have alt attributes', 'Some images missing alt attribute', 1, 1),
('http-status-code', 'http_status_code', 'HTTP Status Code', 'seo', 'field_check', NULL, 1.0, 'HTTP status is OK', 'HTTP error status', 1, 1),
('is-crawlable', 'is_crawlable', 'Page is Crawlable', 'seo', 'selector_count', 'meta[name="robots"]', 1.0, 'Page is crawlable', 'Page has noindex directive', 1, 1);
