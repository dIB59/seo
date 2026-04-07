CREATE TABLE IF NOT EXISTS report_patterns (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'technical',
    severity TEXT NOT NULL DEFAULT 'warning',
    field TEXT NOT NULL,
    operator TEXT NOT NULL,
    threshold TEXT,
    min_prevalence REAL NOT NULL DEFAULT 0.1,
    business_impact TEXT NOT NULL DEFAULT 'medium',
    fix_effort TEXT NOT NULL DEFAULT 'medium',
    recommendation TEXT NOT NULL DEFAULT '',
    is_builtin INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed built-in patterns
INSERT OR IGNORE INTO report_patterns
    (id, name, description, category, severity, field, operator, threshold, min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled)
VALUES
(
    'builtin-missing-meta-desc',
    'Missing Meta Descriptions',
    'Pages without meta descriptions miss out on rich search snippets and have lower click-through rates.',
    'content', 'critical', 'meta_description', 'missing', NULL, 0.1,
    'high', 'low',
    'Add unique, descriptive meta descriptions (150–160 characters) to each page targeting relevant keywords.',
    1, 1
),
(
    'builtin-missing-title',
    'Missing Title Tags',
    'Pages without title tags cannot be properly indexed and ranked by search engines.',
    'content', 'critical', 'title', 'missing', NULL, 0.05,
    'high', 'low',
    'Add a descriptive, keyword-rich title tag (50–60 characters) to every page.',
    1, 1
),
(
    'builtin-thin-content',
    'Thin Content Pages',
    'Pages with very little content are less likely to rank well and provide poor user experience.',
    'content', 'warning', 'word_count', 'lt', '300', 0.2,
    'medium', 'medium',
    'Expand page content to at least 300 words, ensuring the content is relevant, valuable, and addresses user intent.',
    1, 1
),
(
    'builtin-slow-pages',
    'Slow Loading Pages',
    'Pages taking over 3 seconds to load significantly hurt user experience and search rankings.',
    'performance', 'warning', 'load_time_ms', 'gt', '3000', 0.2,
    'high', 'high',
    'Optimise images, enable caching, minimise JavaScript, and use a CDN to reduce page load times below 2 seconds.',
    1, 1
),
(
    'builtin-missing-h1',
    'Missing H1 Headings',
    'Pages without an H1 heading lack a clear primary topic signal for search engines.',
    'content', 'critical', 'h1_count', 'lt', '1', 0.1,
    'high', 'low',
    'Add a single, keyword-focused H1 heading to every page that clearly describes its main topic.',
    1, 1
),
(
    'builtin-multiple-h1',
    'Multiple H1 Headings',
    'Using more than one H1 heading dilutes the primary topic signal and can confuse search engines.',
    'content', 'warning', 'h1_count', 'gt', '1', 0.1,
    'medium', 'low',
    'Ensure each page has exactly one H1 heading. Convert secondary H1s to H2 or H3 headings.',
    1, 1
),
(
    'builtin-missing-canonical',
    'Missing Canonical Tags',
    'Pages without canonical tags risk duplicate content penalties when similar URLs are indexed.',
    'technical', 'warning', 'canonical_url', 'missing', NULL, 0.2,
    'medium', 'low',
    'Add a self-referencing canonical tag to every page to prevent duplicate content issues.',
    1, 1
),
(
    'builtin-missing-viewport',
    'Missing Viewport Meta Tag',
    'Pages without a viewport meta tag are not mobile-friendly, negatively affecting mobile search rankings.',
    'accessibility', 'warning', 'has_viewport', 'eq', 'false', 0.05,
    'high', 'low',
    'Add <meta name="viewport" content="width=device-width, initial-scale=1"> to all page headers.',
    1, 1
),
(
    'builtin-no-structured-data',
    'No Structured Data Markup',
    'Pages without structured data miss rich snippet opportunities in search results.',
    'technical', 'suggestion', 'has_structured_data', 'eq', 'false', 0.9,
    'medium', 'high',
    'Implement relevant Schema.org structured data (e.g. Article, Product, FAQ) to qualify for rich snippets.',
    1, 1
),
(
    'builtin-http-errors',
    'Pages With HTTP Errors',
    'Pages returning 4xx or 5xx status codes waste crawl budget and deliver poor user experience.',
    'technical', 'critical', 'status_code', 'gt', '399', 0.02,
    'high', 'medium',
    'Fix or redirect all error pages. Update all internal links pointing to these URLs.',
    1, 1
);
