-- Add sitemap_found and robots_txt_found to jobs table
ALTER TABLE jobs ADD COLUMN sitemap_found BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE jobs ADD COLUMN robots_txt_found BOOLEAN NOT NULL DEFAULT 0;
