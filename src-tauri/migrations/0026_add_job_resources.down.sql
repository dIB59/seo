-- Remove sitemap_found and robots_txt_found from jobs table
ALTER TABLE jobs DROP COLUMN sitemap_found;
ALTER TABLE jobs DROP COLUMN robots_txt_found;
