-- =============================================================================
-- Seed Benchmark Data for V1 Schema (Old) Performance Testing
-- =============================================================================
-- This script populates the V1 test database with matching data volumes:
-- - ~500 pages across multiple jobs
-- - ~870 issues (various severities)  
-- - ~800 links (page_edge table)
-- =============================================================================

-- First, ensure we have analysis_settings for the jobs
INSERT OR IGNORE INTO analysis_settings (id, max_pages, include_external_links, check_images, mobile_analysis, lighthouse_analysis, delay_between_requests)
VALUES 
    (1, 100, 1, 1, 1, 1, 1000),
    (2, 100, 1, 1, 1, 1, 1000),
    (3, 100, 1, 1, 1, 1, 1000);

-- Create analysis_results for each completed job
INSERT OR REPLACE INTO analysis_results (id, url, status, progress, total_pages, analyzed_pages, sitemap_found, robots_txt_found, ssl_certificate)
SELECT 
    CAST(id AS TEXT),
    url,
    'completed',
    100.0,
    0,
    0,
    1,
    1,
    1
FROM analysis_jobs WHERE status = 'completed';

-- Update analysis_jobs to link to results
UPDATE analysis_jobs SET result_id = CAST(id AS TEXT) WHERE status = 'completed';

-- =============================================================================
-- PAGE_ANALYSIS: Generate 475 pages matching V2 data
-- =============================================================================

-- Job 1: 50 pages
INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
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
    'SEO Insikt - Page ' || n || ' | Professional SEO Services',
    'Meta description for page ' || n || '. We provide professional SEO analysis and optimization services.',
    1,
    (n % 5) + 1,
    (n % 3),
    500 + (n * 37) % 2000,
    (n % 10) + 1,
    (n % 3),
    (n % 15) + 5,
    (n % 5),
    (100 + (n * 17) % 3000) / 1000.0,
    200,
    15000 + (n * 123) % 50000,
    1,
    (n % 5) = 0,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 50);

-- Job 12: 100 pages (larger site)
INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
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
    'Discord - Page ' || n || ' | Your Place to Talk',
    'Discord page ' || n || '. Discord is great for playing games and chilling with friends.',
    1,
    (n % 6) + 1,
    (n % 4),
    800 + (n * 47) % 3000,
    (n % 12) + 1,
    (n % 4),
    (n % 20) + 5,
    (n % 8),
    (50 + (n * 13) % 2000) / 1000.0,
    CASE WHEN n % 20 = 0 THEN 404 WHEN n % 15 = 0 THEN 301 ELSE 200 END,
    20000 + (n * 89) % 80000,
    1,
    (n % 4) = 0,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 100);

-- Job 13: 75 pages
INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
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
    'SEO Insikt V2 - Page ' || n,
    'Updated meta description for SEO Insikt page ' || n || '.',
    1,
    (n % 5) + 1,
    (n % 3),
    600 + (n * 41) % 2500,
    (n % 8) + 1,
    (n % 3),
    (n % 18) + 5,
    (n % 6),
    (80 + (n * 19) % 2500) / 1000.0,
    200,
    18000 + (n * 97) % 60000,
    1,
    (n % 3) = 0,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 75);

-- Jobs 6, 7, 9, 10: 40 pages each
INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
SELECT 
    'page-6-' || printf('%03d', n),
    '6',
    'https://www.seoinsikt.se/archive/page-' || n,
    'Archive Page ' || n,
    'Archived content page ' || n,
    1, 2, 1,
    400 + (n * 29) % 1500,
    5, 1, 10, 2,
    (120 + (n * 11) % 2000) / 1000.0,
    200,
    12000 + (n * 67) % 40000,
    1, 0,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
SELECT 
    'page-7-' || printf('%03d', n),
    '7',
    'https://www.google.com/' || CASE WHEN n = 1 THEN '' ELSE 'search/page-' || n END,
    'Google Search Page ' || n,
    'Google page ' || n,
    1, 1, 1,
    300 + (n * 23) % 1000,
    3, 0, 8, 1,
    (30 + (n * 7) % 500) / 1000.0,
    200,
    8000 + (n * 53) % 30000,
    1, 0,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
SELECT 
    'page-9-' || printf('%03d', n),
    '9',
    'https://www.discord.com/nitro/feature-' || n,
    'Discord Nitro Feature ' || n,
    'Nitro feature page ' || n,
    1, 3, 2,
    700 + (n * 31) % 2000,
    8, 2, 12, 3,
    (60 + (n * 9) % 1500) / 1000.0,
    200,
    16000 + (n * 71) % 50000,
    1, 1,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
SELECT 
    'page-10-' || printf('%03d', n),
    '10',
    'https://www.discord.com/safety/article-' || n,
    'Discord Safety Article ' || n,
    'Safety guidelines page ' || n,
    1, 4, 2,
    900 + (n * 37) % 2500,
    6, 1, 15, 4,
    (70 + (n * 13) % 1800) / 1000.0,
    200,
    19000 + (n * 83) % 55000,
    1, 1,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

-- Jobs 2, 3, 4: 30 pages each
INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
SELECT 
    'page-2-' || printf('%03d', n),
    '2',
    'https://www.somnexperten.se/' || CASE WHEN n = 1 THEN '' ELSE 'produkt/item-' || n END,
    'Somnexperten Product ' || n,
    'Sleep product page ' || n,
    1, 3, 1,
    550 + (n * 27) % 1800,
    7, 2, 11, 3,
    (90 + (n * 11) % 1600) / 1000.0,
    200,
    14000 + (n * 61) % 45000,
    1, 0,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 30);

INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
SELECT 
    'page-3-' || printf('%03d', n),
    '3',
    'https://somnexperten.se/' || CASE WHEN n = 1 THEN '' ELSE 'guide/sleep-' || n END,
    'Sleep Guide ' || n,
    'Sleep advice page ' || n,
    1, 2, 2,
    620 + (n * 33) % 1900,
    5, 1, 9, 2,
    (85 + (n * 14) % 1700) / 1000.0,
    200,
    15000 + (n * 59) % 42000,
    1, 0,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 30);

INSERT OR IGNORE INTO page_analysis (id, analysis_id, url, title, meta_description, h1_count, h2_count, h3_count, word_count, image_count, images_without_alt, internal_links, external_links, load_time, status_code, content_size, mobile_friendly, has_structured_data, created_at)
SELECT 
    'page-4-' || printf('%03d', n),
    '4',
    'https://somnexperten.se/blog/post-' || n,
    'Sleep Blog Post ' || n,
    'Blog post about sleep ' || n,
    1, 4, 3,
    750 + (n * 39) % 2200,
    6, 2, 13, 4,
    (95 + (n * 16) % 1900) / 1000.0,
    200,
    17000 + (n * 73) % 48000,
    1, 1,
    datetime('now')
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 30);

-- =============================================================================
-- SEO_ISSUES: Generate ~870 issues matching V2 data
-- =============================================================================

-- Job 12 issues (200 issues)
INSERT OR IGNORE INTO seo_issues (id, page_id, type, title, description, page_url, recommendation)
SELECT 
    'issue-12-' || printf('%04d', n),
    'page-12-' || printf('%03d', ((n-1) % 100) + 1),
    CASE 
        WHEN (n % 11) < 4 THEN 'critical'
        WHEN (n % 11) < 8 THEN 'warning'
        ELSE 'suggestion'
    END,
    CASE (n % 11)
        WHEN 0 THEN 'Missing Title Tag'
        WHEN 1 THEN 'Missing H1 Heading'
        WHEN 2 THEN 'Broken Link Detected'
        WHEN 3 THEN 'Slow Page Load Time'
        WHEN 4 THEN 'Short Meta Description'
        WHEN 5 THEN 'Duplicate Title Tag'
        WHEN 6 THEN 'Missing Alt Text'
        WHEN 7 THEN 'Redirect Chain'
        WHEN 8 THEN 'Title Too Long'
        WHEN 9 THEN 'Orphan Page'
        ELSE 'Low Word Count'
    END,
    'Issue ' || n || ' detected on page ' || (((n-1) % 100) + 1),
    'https://www.discord.com/page-' || (((n-1) % 100) + 1),
    'Fix this SEO issue to improve rankings'
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs, analysis_jobs LIMIT 200);

-- Job 13 issues (150 issues)
INSERT OR IGNORE INTO seo_issues (id, page_id, type, title, description, page_url, recommendation)
SELECT 
    'issue-13-' || printf('%04d', n),
    'page-13-' || printf('%03d', ((n-1) % 75) + 1),
    CASE 
        WHEN (n % 11) < 4 THEN 'critical'
        WHEN (n % 11) < 8 THEN 'warning'
        ELSE 'suggestion'
    END,
    CASE (n % 11)
        WHEN 0 THEN 'Missing Title Tag'
        WHEN 1 THEN 'Missing H1 Heading'
        WHEN 2 THEN 'Broken Link Detected'
        WHEN 3 THEN 'Slow Page Load Time'
        WHEN 4 THEN 'Short Meta Description'
        WHEN 5 THEN 'Duplicate Title Tag'
        WHEN 6 THEN 'Missing Alt Text'
        WHEN 7 THEN 'Redirect Chain'
        WHEN 8 THEN 'Title Too Long'
        WHEN 9 THEN 'Orphan Page'
        ELSE 'Low Word Count'
    END,
    'Issue ' || n || ' found on page ' || (((n-1) % 75) + 1),
    'https://www.seoinsikt.se/v2/page-' || (((n-1) % 75) + 1),
    'Address this issue for better SEO'
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs, analysis_jobs LIMIT 150);

-- Job 1 issues (100 issues)
INSERT OR IGNORE INTO seo_issues (id, page_id, type, title, description, page_url, recommendation)
SELECT 
    'issue-1-' || printf('%04d', n),
    'page-1-' || printf('%03d', ((n-1) % 50) + 1),
    CASE 
        WHEN (n % 8) < 3 THEN 'critical'
        WHEN (n % 8) < 5 THEN 'warning'
        ELSE 'suggestion'
    END,
    CASE (n % 8)
        WHEN 0 THEN 'Missing Title Tag'
        WHEN 1 THEN 'Broken Link Detected'
        WHEN 2 THEN 'Slow Page Load Time'
        WHEN 3 THEN 'Short Meta Description'
        WHEN 4 THEN 'Missing Alt Text'
        WHEN 5 THEN 'Title Too Long'
        WHEN 6 THEN 'Low Word Count'
        ELSE 'Orphan Page'
    END,
    'SEO issue ' || n || ' on seoinsikt.se page',
    'https://www.seoinsikt.se/page-' || (((n-1) % 50) + 1),
    'Improve this for better rankings'
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 100);

-- Issues for other jobs (60 each for jobs 2, 4, 6, 9)
INSERT OR IGNORE INTO seo_issues (id, page_id, type, title, description, page_url, recommendation)
SELECT 
    'issue-2-' || printf('%04d', n),
    'page-2-' || printf('%03d', ((n-1) % 30) + 1),
    CASE WHEN (n % 7) < 2 THEN 'critical' WHEN (n % 7) < 4 THEN 'warning' ELSE 'suggestion' END,
    'SEO Issue Type ' || (n % 7),
    'Issue ' || n || ' description',
    'https://www.somnexperten.se/page-' || n,
    'Fix recommendation'
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 60);

INSERT OR IGNORE INTO seo_issues (id, page_id, type, title, description, page_url, recommendation)
SELECT 
    'issue-4-' || printf('%04d', n),
    'page-4-' || printf('%03d', ((n-1) % 30) + 1),
    CASE WHEN (n % 7) < 2 THEN 'critical' WHEN (n % 7) < 4 THEN 'warning' ELSE 'suggestion' END,
    'SEO Issue Type ' || (n % 7),
    'Issue ' || n || ' description',
    'https://somnexperten.se/blog/post-' || n,
    'Fix recommendation'
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 60);

INSERT OR IGNORE INTO seo_issues (id, page_id, type, title, description, page_url, recommendation)
SELECT 
    'issue-6-' || printf('%04d', n),
    'page-6-' || printf('%03d', ((n-1) % 40) + 1),
    CASE WHEN (n % 7) < 2 THEN 'critical' WHEN (n % 7) < 4 THEN 'warning' ELSE 'suggestion' END,
    'SEO Issue Type ' || (n % 7),
    'Issue ' || n || ' description',
    'https://www.seoinsikt.se/archive/page-' || n,
    'Fix recommendation'
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 60);

INSERT OR IGNORE INTO seo_issues (id, page_id, type, title, description, page_url, recommendation)
SELECT 
    'issue-9-' || printf('%04d', n),
    'page-9-' || printf('%03d', ((n-1) % 40) + 1),
    CASE WHEN (n % 7) < 2 THEN 'critical' WHEN (n % 7) < 4 THEN 'warning' ELSE 'suggestion' END,
    'SEO Issue Type ' || (n % 7),
    'Issue ' || n || ' description',
    'https://www.discord.com/nitro/feature-' || n,
    'Fix recommendation'
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 60);

-- =============================================================================
-- PAGE_EDGE: Generate link graph (~800 links)
-- =============================================================================

-- Job 12 edges (100 links)
INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 
    'page-12-' || printf('%03d', ((n-1) % 100) + 1),
    'https://www.discord.com/page-' || (((n-1) * 7 + 3) % 100),
    CASE WHEN (n % 25) = 0 THEN 404 WHEN (n % 30) = 0 THEN 301 ELSE 200 END
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 100);

-- Job 13 edges (375 links)
INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 
    'page-13-' || printf('%03d', ((n-1) % 75) + 1),
    'https://www.seoinsikt.se/v2/page-' || (((n-1) * 11 + 5) % 75),
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs, analysis_jobs LIMIT 375);

-- Job 1 edges (70 links)
INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 
    'page-1-' || printf('%03d', ((n-1) % 50) + 1),
    'https://www.seoinsikt.se/page-' || (((n-1) * 13 + 7) % 50),
    200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs, analysis_jobs LIMIT 70);

-- Other jobs edges (30-40 each)
INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 'page-2-' || printf('%03d', ((n-1) % 30) + 1), 'https://www.somnexperten.se/page-' || n, 200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 30);

INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 'page-3-' || printf('%03d', ((n-1) % 30) + 1), 'https://somnexperten.se/page-' || n, 200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 30);

INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 'page-4-' || printf('%03d', ((n-1) % 30) + 1), 'https://somnexperten.se/blog/page-' || n, 200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 30);

INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 'page-6-' || printf('%03d', ((n-1) % 40) + 1), 'https://www.seoinsikt.se/archive/page-' || n, 200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 'page-7-' || printf('%03d', ((n-1) % 40) + 1), 'https://www.google.com/page-' || n, 200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 'page-9-' || printf('%03d', ((n-1) % 40) + 1), 'https://www.discord.com/nitro/page-' || n, 200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

INSERT OR IGNORE INTO page_edge (from_page_id, to_url, status_code)
SELECT 'page-10-' || printf('%03d', ((n-1) % 40) + 1), 'https://www.discord.com/safety/page-' || n, 200
FROM (SELECT ROW_NUMBER() OVER () as n FROM analysis_jobs, analysis_jobs LIMIT 40);

-- =============================================================================
-- UPDATE ANALYSIS_RESULTS with page counts
-- =============================================================================

UPDATE analysis_results SET 
    total_pages = (SELECT COUNT(*) FROM page_analysis WHERE analysis_id = analysis_results.id),
    analyzed_pages = (SELECT COUNT(*) FROM page_analysis WHERE analysis_id = analysis_results.id)
WHERE id IN ('1', '2', '3', '4', '6', '7', '9', '10', '12', '13');

-- Apply pragmas for benchmarking
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -65536;

-- Verify counts
SELECT 'Pages: ' || COUNT(*) FROM page_analysis;
SELECT 'Issues: ' || COUNT(*) FROM seo_issues;
SELECT 'Edges: ' || COUNT(*) FROM page_edge;
