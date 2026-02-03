-- Drop composite index for edge lookups by source and target
DROP INDEX IF EXISTS ix_page_edge_from_to;

-- Drop index for reverse lookups on edges (to_url)
DROP INDEX IF EXISTS ix_page_edge_to_url;
