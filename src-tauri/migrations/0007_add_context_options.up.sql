-- Add context options setting
INSERT INTO settings (key, value) VALUES 
('gemini_context_options', '["url","score","pages","issues","metrics","ssl","sitemap","robots"]')
ON CONFLICT(key) DO NOTHING;
