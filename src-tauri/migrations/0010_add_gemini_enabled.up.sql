INSERT INTO settings (key, value) VALUES ('gemini_enabled', 'true') ON CONFLICT(key) DO NOTHING;
