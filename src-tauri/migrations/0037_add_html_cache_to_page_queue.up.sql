-- Cache fetched HTML in the page queue so the analysis phase can skip
-- re-fetching pages that were already visited during discovery.
-- All columns are nullable for backwards compat with existing rows.
-- cached_html is NULLed after analysis completes to free space.

ALTER TABLE page_queue ADD COLUMN cached_html TEXT;
ALTER TABLE page_queue ADD COLUMN http_status INTEGER;
ALTER TABLE page_queue ADD COLUMN cached_load_time_ms REAL;
ALTER TABLE page_queue ADD COLUMN final_url TEXT;
