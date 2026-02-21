-- Migration 0025: Remove Missing Gemini Settings (Down)
-- -----------------------------------------------------------------------------

ALTER TABLE settings DROP COLUMN gemini_enabled;
ALTER TABLE settings DROP COLUMN gemini_persona;
ALTER TABLE settings DROP COLUMN gemini_requirements;
ALTER TABLE settings DROP COLUMN gemini_context_options;
ALTER TABLE settings DROP COLUMN gemini_prompt_blocks;

-- Update updated_at
UPDATE settings SET updated_at = datetime('now') WHERE id = 1;
