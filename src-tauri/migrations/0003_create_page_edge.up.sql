CREATE TABLE page_edge (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    from_page_id  TEXT NOT NULL,
    to_url        TEXT NOT NULL,
    status_code   INTEGER NOT NULL,
    FOREIGN KEY (from_page_id) REFERENCES page_analysis(id) ON DELETE CASCADE
);

-- Speed up look-ups by origin page
CREATE INDEX idx_edge_from ON page_edge(from_page_id);
