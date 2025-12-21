-- Index for reverse lookups on edges - finding what pages link TO a URL
-- Used for broken link detection and backlink analysis
CREATE INDEX IF NOT EXISTS ix_page_edge_to_url 
ON page_edge(to_url);

-- Composite index for edge lookups by both source and target
-- Improves performance when checking if specific link exists
CREATE INDEX IF NOT EXISTS ix_page_edge_from_to 
ON page_edge(from_page_id, to_url);
