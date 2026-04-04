mod content;
mod seo;

use crate::checker::Check;

/// Returns all built-in checks.
pub fn all() -> Vec<Box<dyn Check>> {
    vec![
        // SEO checks derived from SeoAuditDetails
        Box::new(seo::HttpStatusCheck),
        Box::new(seo::CrawlabilityCheck),
        Box::new(seo::TitleCheck),
        Box::new(seo::MetaDescriptionCheck),
        Box::new(seo::ViewportCheck),
        Box::new(seo::CanonicalCheck),
        Box::new(seo::ImageAltCheck),
        Box::new(seo::LinkTextCheck),
        Box::new(seo::CrawlableAnchorsCheck),
        // Content checks derived from Page fields
        Box::new(content::WordCountCheck),
        Box::new(content::LoadTimeCheck),
    ]
}
