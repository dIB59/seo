-- Create settings table for storing application configuration
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default placeholder for Gemini API key
INSERT INTO settings (key, value) VALUES ('gemini_api_key', '') 
ON CONFLICT(key) DO NOTHING;
