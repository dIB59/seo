-- Migration 0024: Add Licensing and Missing Settings Columns
-- -----------------------------------------------------------------------------

-- Licensing
ALTER TABLE settings ADD COLUMN signed_license TEXT;

-- Update updated_at
UPDATE settings SET updated_at = datetime('now') WHERE id = 1;
