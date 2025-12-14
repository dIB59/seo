-- Add prompt blocks setting
INSERT INTO settings (key, value) VALUES 
('gemini_prompt_blocks', '[
    {"id": "intro", "type": "text", "content": "Please provide:\n1. A brief executive summary of the site''s SEO health (2-3 sentences)\n2. Top 5 priority actions the site owner should take, ranked by impact\n3. Expected outcomes if these recommendations are implemented\n\nKeep your response concise, actionable, and professional. Format for a PDF report.\n\nData to include:"},
    {"id": "url", "type": "variable", "content": "Website URL: {url}"},
    {"id": "score", "type": "variable", "content": "SEO Score: {score}"},
    {"id": "pages", "type": "variable", "content": "Pages Analyzed: {pages_count}"},
    {"id": "issues", "type": "variable", "content": "Total Issues Breakdown: {total_issues}"},
    {"id": "top_issues", "type": "variable", "content": "Top 5 Issues List: {top_issues}"},
    {"id": "metrics", "type": "variable", "content": "Site Metrics: {avg_load_time}"},
    {"id": "ssl", "type": "variable", "content": "SSL Status: {ssl_certificate}"},
    {"id": "sitemap", "type": "variable", "content": "Sitemap Status: {sitemap_found}"},
    {"id": "robots", "type": "variable", "content": "Robots.txt Status: {robots_txt_found}"}
]')
ON CONFLICT(key) DO NOTHING;
