-- =============================================================================
-- Seed Benchmark Data for V2 Schema Performance Testing
-- =============================================================================
-- This script populates the test database with realistic data volumes:
-- - ~500 pages across multiple jobs
-- - ~2000 issues (various severities)  
-- - ~3000 links (internal link graph)
-- =============================================================================

-- Disable triggers temporarily for faster bulk inserts
-- (We'll update stats manually at the end)
PRAGMA defer_foreign_keys = ON;

-- =============================================================================
-- PAGES: Generate 50 pages per completed job (jobs 1,2,3,4,6,7,9,10,12,13)
-- =============================================================================

-- Job 1: https://www.seoinsikt.se/ - 50 pages
INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-1-' || printf('%03d', n),
    '1',
    'https://www.seoinsikt.se/' || CASE 
        WHEN n = 1 THEN ''
        WHEN n <= 5 THEN 'tjanster/service-' || (n-1)
        WHEN n <= 15 THEN 'blogg/article-' || (n-5)
        WHEN n <= 25 THEN 'produkter/product-' || (n-15)
        WHEN n <= 35 THEN 'om-oss/team/person-' || (n-25)
        ELSE 'sidor/page-' || (n-35)
    END,
    CASE WHEN n = 1 THEN 0 WHEN n <= 5 THEN 1 WHEN n <= 35 THEN 2 ELSE 3 END,
    200,
    'text/html',
    'SEO Insikt - Page ' || n || ' | Professional SEO Services',
    'Meta description for page ' || n || '. We provide professional SEO analysis and optimization services.',
    500 + (n * 37) % 2000,
    100 + (n * 17) % 3000,
    15000 + (n * 123) % 50000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 50);

-- Job 12: https://www.discord.com/ - 100 pages (larger site)
INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-12-' || printf('%03d', n),
    '12',
    'https://www.discord.com/' || CASE 
        WHEN n = 1 THEN ''
        WHEN n <= 10 THEN 'features/feature-' || (n-1)
        WHEN n <= 30 THEN 'blog/post-' || (n-10)
        WHEN n <= 50 THEN 'support/article-' || (n-30)
        WHEN n <= 70 THEN 'community/topic-' || (n-50)
        WHEN n <= 90 THEN 'developers/docs/page-' || (n-70)
        ELSE 'misc/page-' || (n-90)
    END,
    CASE WHEN n = 1 THEN 0 WHEN n <= 10 THEN 1 WHEN n <= 50 THEN 2 ELSE 3 END,
    CASE WHEN n % 20 = 0 THEN 404 WHEN n % 15 = 0 THEN 301 ELSE 200 END,
    'text/html',
    'Discord - Page ' || n || ' | Your Place to Talk',
    'Discord page ' || n || '. Discord is great for playing games and chilling with friends.',
    800 + (n * 47) % 3000,
    50 + (n * 13) % 2000,
    20000 + (n * 89) % 80000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 100);

-- Job 13: https://www.seoinsikt.se/ - 75 pages
INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-13-' || printf('%03d', n),
    '13',
    'https://www.seoinsikt.se/v2/' || CASE 
        WHEN n = 1 THEN ''
        WHEN n <= 8 THEN 'services/svc-' || (n-1)
        WHEN n <= 20 THEN 'blog/post-' || (n-8)
        WHEN n <= 40 THEN 'case-studies/case-' || (n-20)
        WHEN n <= 55 THEN 'resources/guide-' || (n-40)
        ELSE 'pages/misc-' || (n-55)
    END,
    CASE WHEN n = 1 THEN 0 WHEN n <= 8 THEN 1 WHEN n <= 40 THEN 2 ELSE 3 END,
    200,
    'text/html',
    'SEO Insikt V2 - Page ' || n,
    'Updated meta description for SEO Insikt page ' || n || '.',
    600 + (n * 41) % 2500,
    80 + (n * 19) % 2500,
    18000 + (n * 97) % 60000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 75);

-- Job 6, 7, 9, 10: Add 40 pages each
INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-6-' || printf('%03d', n),
    '6',
    'https://www.seoinsikt.se/archive/' || 'page-' || n,
    (n % 4),
    200,
    'text/html',
    'Archive Page ' || n,
    'Archived content page ' || n,
    400 + (n * 29) % 1500,
    120 + (n * 11) % 2000,
    12000 + (n * 67) % 40000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs LIMIT 40);

INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-7-' || printf('%03d', n),
    '7',
    'https://www.google.com/' || CASE WHEN n = 1 THEN '' ELSE 'search/page-' || n END,
    (n % 3),
    200,
    'text/html',
    'Google Search Page ' || n,
    'Google page ' || n,
    300 + (n * 23) % 1000,
    30 + (n * 7) % 500,
    8000 + (n * 53) % 30000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs LIMIT 40);

INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-9-' || printf('%03d', n),
    '9',
    'https://www.discord.com/nitro/' || 'feature-' || n,
    (n % 4),
    200,
    'text/html',
    'Discord Nitro Feature ' || n,
    'Nitro feature page ' || n,
    700 + (n * 31) % 2000,
    60 + (n * 9) % 1500,
    16000 + (n * 71) % 50000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs LIMIT 40);

INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-10-' || printf('%03d', n),
    '10',
    'https://www.discord.com/safety/' || 'article-' || n,
    (n % 3),
    200,
    'text/html',
    'Discord Safety Article ' || n,
    'Safety guidelines page ' || n,
    900 + (n * 37) % 2500,
    70 + (n * 13) % 1800,
    19000 + (n * 83) % 55000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs LIMIT 40);

-- Jobs 2,3,4: Add 30 pages each
INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-2-' || printf('%03d', n),
    '2',
    'https://www.somnexperten.se/' || CASE WHEN n = 1 THEN '' ELSE 'produkt/item-' || n END,
    (n % 3),
    200,
    'text/html',
    'Somnexperten Product ' || n,
    'Sleep product page ' || n,
    550 + (n * 27) % 1800,
    90 + (n * 11) % 1600,
    14000 + (n * 61) % 45000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs LIMIT 30);

INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-3-' || printf('%03d', n),
    '3',
    'https://somnexperten.se/' || CASE WHEN n = 1 THEN '' ELSE 'guide/sleep-' || n END,
    (n % 3),
    200,
    'text/html',
    'Sleep Guide ' || n,
    'Sleep advice page ' || n,
    620 + (n * 33) % 1900,
    85 + (n * 14) % 1700,
    15000 + (n * 59) % 42000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs LIMIT 30);

INSERT OR IGNORE INTO pages (id, job_id, url, depth, status_code, content_type, title, meta_description, word_count, load_time_ms, response_size_bytes)
SELECT 
    'page-4-' || printf('%03d', n),
    '4',
    'https://somnexperten.se/blog/' || 'post-' || n,
    (n % 4),
    200,
    'text/html',
    'Sleep Blog Post ' || n,
    'Blog post about sleep ' || n,
    750 + (n * 39) % 2200,
    95 + (n * 16) % 1900,
    17000 + (n * 73) % 48000
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs LIMIT 30);

-- =============================================================================
-- ISSUES: Generate realistic SEO issues for each job
-- =============================================================================

-- Common issue types and their distribution
-- Critical: missing_title, missing_h1, broken_link, slow_page
-- Warning: short_description, duplicate_title, missing_alt, redirect_chain
-- Info: long_title, orphan_page, low_word_count

-- Job 12 issues (100 pages = ~200 issues)
INSERT OR IGNORE INTO issues (job_id, page_id, type, severity, message, details)
SELECT 
    '12',
    'page-12-' || printf('%03d', ((n-1) % 100) + 1),
    CASE (n % 11)
        WHEN 0 THEN 'missing_title'
        WHEN 1 THEN 'missing_h1'
        WHEN 2 THEN 'broken_link'
        WHEN 3 THEN 'slow_page'
        WHEN 4 THEN 'short_description'
        WHEN 5 THEN 'duplicate_title'
        WHEN 6 THEN 'missing_alt'
        WHEN 7 THEN 'redirect_chain'
        WHEN 8 THEN 'long_title'
        WHEN 9 THEN 'orphan_page'
        ELSE 'low_word_count'
    END,
    CASE 
        WHEN (n % 11) < 4 THEN 'critical'
        WHEN (n % 11) < 8 THEN 'warning'
        ELSE 'info'
    END,
    'Issue ' || n || ' detected on page ' || (((n-1) % 100) + 1),
    '{"value": ' || n || ', "threshold": 100}'
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs, jobs LIMIT 200);

-- Job 13 issues (75 pages = ~150 issues)
INSERT OR IGNORE INTO issues (job_id, page_id, type, severity, message, details)
SELECT 
    '13',
    'page-13-' || printf('%03d', ((n-1) % 75) + 1),
    CASE (n % 11)
        WHEN 0 THEN 'missing_title'
        WHEN 1 THEN 'missing_h1'
        WHEN 2 THEN 'broken_link'
        WHEN 3 THEN 'slow_page'
        WHEN 4 THEN 'short_description'
        WHEN 5 THEN 'duplicate_title'
        WHEN 6 THEN 'missing_alt'
        WHEN 7 THEN 'redirect_chain'
        WHEN 8 THEN 'long_title'
        WHEN 9 THEN 'orphan_page'
        ELSE 'low_word_count'
    END,
    CASE 
        WHEN (n % 11) < 4 THEN 'critical'
        WHEN (n % 11) < 8 THEN 'warning'
        ELSE 'info'
    END,
    'Issue ' || n || ' found on page ' || (((n-1) % 75) + 1),
    '{"detected": true, "score": ' || (n % 100) || '}'
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs, jobs LIMIT 150);

-- Job 1 issues (50 pages = ~100 issues)
INSERT OR IGNORE INTO issues (job_id, page_id, type, severity, message, details)
SELECT 
    '1',
    'page-1-' || printf('%03d', ((n-1) % 50) + 1),
    CASE (n % 8)
        WHEN 0 THEN 'missing_title'
        WHEN 1 THEN 'broken_link'
        WHEN 2 THEN 'slow_page'
        WHEN 3 THEN 'short_description'
        WHEN 4 THEN 'missing_alt'
        WHEN 5 THEN 'long_title'
        WHEN 6 THEN 'low_word_count'
        ELSE 'orphan_page'
    END,
    CASE 
        WHEN (n % 8) < 3 THEN 'critical'
        WHEN (n % 8) < 5 THEN 'warning'
        ELSE 'info'
    END,
    'SEO issue ' || n || ' on seoinsikt.se page',
    NULL
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 100);

-- Issues for other jobs (jobs 2,3,4,6,7,9,10)
INSERT OR IGNORE INTO issues (job_id, page_id, type, severity, message, details)
SELECT 
    CAST(((n-1) / 60 % 7) + CASE ((n-1) / 60 % 7) WHEN 0 THEN 2 WHEN 1 THEN 3 WHEN 2 THEN 4 WHEN 3 THEN 6 WHEN 4 THEN 7 WHEN 5 THEN 9 ELSE 10 END AS TEXT),
    'page-' || (CASE ((n-1) / 60 % 7) WHEN 0 THEN 2 WHEN 1 THEN 3 WHEN 2 THEN 4 WHEN 3 THEN 6 WHEN 4 THEN 7 WHEN 5 THEN 9 ELSE 10 END) || '-' || printf('%03d', ((n-1) % 30) + 1),
    CASE (n % 7)
        WHEN 0 THEN 'missing_title'
        WHEN 1 THEN 'slow_page'
        WHEN 2 THEN 'short_description'
        WHEN 3 THEN 'missing_alt'
        WHEN 4 THEN 'long_title'
        WHEN 5 THEN 'low_word_count'
        ELSE 'redirect_chain'
    END,
    CASE 
        WHEN (n % 7) < 2 THEN 'critical'
        WHEN (n % 7) < 4 THEN 'warning'
        ELSE 'info'
    END,
    'Detected issue ' || n,
    NULL
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs, jobs LIMIT 420);

-- =============================================================================
-- LINKS: Generate internal link graph
-- =============================================================================

-- Job 12 links (100 pages, ~15-20 links per page = ~1500 links)
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '12',
    'page-12-' || printf('%03d', ((n-1) % 100) + 1),
    'page-12-' || printf('%03d', (((n-1) * 7 + 3) % 100) + 1),
    'https://www.discord.com/' || CASE 
        WHEN (n % 5) = 0 THEN 'features/feature-' || ((n % 10) + 1)
        WHEN (n % 5) = 1 THEN 'blog/post-' || ((n % 20) + 1)
        WHEN (n % 5) = 2 THEN 'support/article-' || ((n % 20) + 1)
        WHEN (n % 5) = 3 THEN 'community/topic-' || ((n % 20) + 1)
        ELSE 'developers/docs/page-' || ((n % 20) + 1)
    END,
    'Link text ' || n,
    CASE WHEN (n % 10) = 0 THEN 'external' ELSE 'internal' END,
    CASE WHEN (n % 20) = 0 THEN 0 ELSE 1 END,
    CASE WHEN (n % 25) = 0 THEN 404 WHEN (n % 30) = 0 THEN 301 ELSE 200 END
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs, jobs, jobs LIMIT 1500);

-- Job 13 links (75 pages, ~12 links per page = ~900 links)
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '13',
    'page-13-' || printf('%03d', ((n-1) % 75) + 1),
    'page-13-' || printf('%03d', (((n-1) * 11 + 5) % 75) + 1),
    'https://www.seoinsikt.se/v2/' || CASE 
        WHEN (n % 4) = 0 THEN 'services/svc-' || ((n % 8) + 1)
        WHEN (n % 4) = 1 THEN 'blog/post-' || ((n % 12) + 1)
        WHEN (n % 4) = 2 THEN 'case-studies/case-' || ((n % 20) + 1)
        ELSE 'resources/guide-' || ((n % 15) + 1)
    END,
    'Navigation link ' || n,
    CASE WHEN (n % 12) = 0 THEN 'external' ELSE 'internal' END,
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs, jobs LIMIT 900);

-- Job 1 links (50 pages, ~10 links per page = ~500 links)
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '1',
    'page-1-' || printf('%03d', ((n-1) % 50) + 1),
    'page-1-' || printf('%03d', (((n-1) * 13 + 7) % 50) + 1),
    'https://www.seoinsikt.se/' || CASE 
        WHEN (n % 5) = 0 THEN 'tjanster/service-' || ((n % 5) + 1)
        WHEN (n % 5) = 1 THEN 'blogg/article-' || ((n % 10) + 1)
        WHEN (n % 5) = 2 THEN 'produkter/product-' || ((n % 10) + 1)
        WHEN (n % 5) = 3 THEN 'om-oss/team/person-' || ((n % 10) + 1)
        ELSE 'sidor/page-' || ((n % 15) + 1)
    END,
    'Internal link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs, jobs LIMIT 500);

-- Links for other jobs (200 links each for jobs 2,3,4,6,7,9,10)
-- Job 2
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '2',
    'page-2-' || printf('%03d', ((n-1) % 30) + 1),
    'page-2-' || printf('%03d', (((n-1) * 7 + 3) % 30) + 1),
    'https://www.somnexperten.se/produkt/item-' || ((n % 30) + 1),
    'Product link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 200);

-- Job 3
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '3',
    'page-3-' || printf('%03d', ((n-1) % 30) + 1),
    'page-3-' || printf('%03d', (((n-1) * 11 + 5) % 30) + 1),
    'https://somnexperten.se/guide/sleep-' || ((n % 30) + 1),
    'Guide link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 200);

-- Job 4
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '4',
    'page-4-' || printf('%03d', ((n-1) % 30) + 1),
    'page-4-' || printf('%03d', (((n-1) * 13 + 7) % 30) + 1),
    'https://somnexperten.se/blog/post-' || ((n % 30) + 1),
    'Blog link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 200);

-- Job 6
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '6',
    'page-6-' || printf('%03d', ((n-1) % 40) + 1),
    'page-6-' || printf('%03d', (((n-1) * 17 + 11) % 40) + 1),
    'https://www.seoinsikt.se/archive/page-' || ((n % 40) + 1),
    'Archive link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 200);

-- Job 7
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '7',
    'page-7-' || printf('%03d', ((n-1) % 40) + 1),
    'page-7-' || printf('%03d', (((n-1) * 19 + 13) % 40) + 1),
    'https://www.google.com/search/page-' || ((n % 40) + 1),
    'Search link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 200);

-- Job 9
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '9',
    'page-9-' || printf('%03d', ((n-1) % 40) + 1),
    'page-9-' || printf('%03d', (((n-1) * 23 + 17) % 40) + 1),
    'https://www.discord.com/nitro/feature-' || ((n % 40) + 1),
    'Nitro link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 200);

-- Job 10
INSERT OR IGNORE INTO links (job_id, source_page_id, target_page_id, target_url, link_text, link_type, is_followed, status_code)
SELECT 
    '10',
    'page-10-' || printf('%03d', ((n-1) % 40) + 1),
    'page-10-' || printf('%03d', (((n-1) * 29 + 19) % 40) + 1),
    'https://www.discord.com/safety/article-' || ((n % 40) + 1),
    'Safety link ' || n,
    'internal',
    1,
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM jobs, jobs, jobs LIMIT 200);

-- =============================================================================
-- UPDATE JOB STATS (since triggers may have been disabled)
-- =============================================================================

UPDATE jobs SET 
    total_pages = (SELECT COUNT(*) FROM pages WHERE pages.job_id = jobs.id),
    pages_crawled = (SELECT COUNT(*) FROM pages WHERE pages.job_id = jobs.id),
    total_issues = (SELECT COUNT(*) FROM issues WHERE issues.job_id = jobs.id),
    critical_issues = (SELECT COUNT(*) FROM issues WHERE issues.job_id = jobs.id AND severity = 'critical'),
    warning_issues = (SELECT COUNT(*) FROM issues WHERE issues.job_id = jobs.id AND severity = 'warning'),
    info_issues = (SELECT COUNT(*) FROM issues WHERE issues.job_id = jobs.id AND severity = 'info'),
    progress = 100.0,
    status = 'completed'
WHERE id IN ('1', '2', '3', '4', '6', '7', '9', '10', '12', '13');

-- Verify counts
SELECT 'Pages: ' || COUNT(*) FROM pages;
SELECT 'Issues: ' || COUNT(*) FROM issues;
SELECT 'Links: ' || COUNT(*) FROM links;
