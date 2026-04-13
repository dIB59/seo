-- Reverse of 0034: rename tag back to key and rewrite `tag:` field
-- prefixes back to `extracted:` on both custom_checks and report_patterns.

UPDATE report_patterns
SET    field = 'extracted:' || substr(field, length('tag:') + 1)
WHERE  field LIKE 'tag:%';

UPDATE custom_checks
SET    field = 'extracted:' || substr(field, length('tag:') + 1)
WHERE  field LIKE 'tag:%';

ALTER TABLE custom_extractors RENAME COLUMN tag TO key;
