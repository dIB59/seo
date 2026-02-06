-- Add lighthouse_analysis flag to jobs settings
ALTER TABLE jobs ADD COLUMN lighthouse_analysis INTEGER NOT NULL DEFAULT 0;
