-- Rename custom_extractors.key → custom_extractors.tag, and migrate
-- every custom_checks.field / report_patterns.field that references a
-- custom extractor via the legacy `extracted:<key>` prefix to the new
-- canonical `tag:<key>` form.
--
-- This is chunk 1 of the "tags" feature: the extractor-defined symbol
-- users can reference in checks and (later) report templates gets a
-- dedicated name across the whole stack.

ALTER TABLE custom_extractors RENAME COLUMN key TO tag;

UPDATE custom_checks
SET    field = 'tag:' || substr(field, length('extracted:') + 1)
WHERE  field LIKE 'extracted:%';

UPDATE report_patterns
SET    field = 'tag:' || substr(field, length('extracted:') + 1)
WHERE  field LIKE 'extracted:%';
