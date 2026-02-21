-- Migration 0024: Down
-- -----------------------------------------------------------------------------

ALTER TABLE settings DROP COLUMN gemini_enabled;
ALTER TABLE settings DROP COLUMN gemini_persona;
ALTER TABLE settings DROP COLUMN gemini_requirements;
ALTER TABLE settings DROP COLUMN gemini_context_options;
ALTER TABLE settings DROP COLUMN gemini_prompt_blocks;
ALTER TABLE settings DROP COLUMN signed_license;
