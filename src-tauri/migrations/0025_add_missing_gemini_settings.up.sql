-- Migration 0025: Add Missing Gemini Settings
-- -----------------------------------------------------------------------------

ALTER TABLE settings ADD COLUMN gemini_enabled INTEGER DEFAULT 0;
ALTER TABLE settings ADD COLUMN gemini_persona TEXT;
ALTER TABLE settings ADD COLUMN gemini_requirements TEXT;
ALTER TABLE settings ADD COLUMN gemini_context_options TEXT;
ALTER TABLE settings ADD COLUMN gemini_prompt_blocks TEXT;

-- Update updated_at
UPDATE settings SET updated_at = datetime('now') WHERE id = 1;
