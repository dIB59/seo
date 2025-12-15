CREATE TABLE IF NOT EXISTS analysis_ai_insights (
    analysis_id TEXT PRIMARY KEY,
    insights TEXT NOT NULL,
    created_at DATETIME DEFAULT (datetime('now')),
    FOREIGN KEY (analysis_id) REFERENCES analysis_results(id) ON DELETE CASCADE
);
