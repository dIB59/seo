-- Add columns for storing detailed page data as JSON
ALTER TABLE page_analysis ADD COLUMN headings TEXT;
ALTER TABLE page_analysis ADD COLUMN images TEXT;
ALTER TABLE page_analysis ADD COLUMN links TEXT;
