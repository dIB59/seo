use std::collections::HashMap;

use crate::contexts::analysis::{CompleteJobResult, LighthouseData, Page};

use super::dto::*;

fn count_links_by_type(links: &[LinkDetail]) -> (i64, i64) {
    links.iter().fold((0, 0), |(internal, external), link| {
        if link.link_type == crate::contexts::analysis::LinkType::Internal {
            (internal + 1, external)
        } else {
            (internal, external + 1)
        }
    })
}

fn count_images_without_alt(images: &[ImageElement]) -> i64 {
    images
        .iter()
        .filter(|img| img.alt.as_deref().unwrap_or("").is_empty())
        .count() as i64
}

fn assemble_page(
    page: Page,
    lh_data: Option<&LighthouseData>,
    detailed_links: Vec<LinkDetail>,
    headings: Vec<HeadingElement>,
    images: Vec<ImageElement>,
    extracted_data: std::collections::HashMap<String, serde_json::Value>,
) -> PageAnalysisData {
    let load_time = page.load_time_ms.unwrap_or(0) as f64 / 1000.0;
    let lh_interpreted = lh_data.map(|lh| lh.interpret()).unwrap_or_default();
    let mobile_friendly = lh_interpreted.mobile_friendly || page.is_mobile_friendly_heuristic();
    let has_structured_data = page.has_structured_data || lh_interpreted.has_structured_data;
    let lighthouse_seo_audits = lh_interpreted.seo_audits;
    let lighthouse_performance_metrics = lh_interpreted.performance_metrics;
    let (internal_links, external_links) = count_links_by_type(&detailed_links);

    PageAnalysisData {
        analysis_id: page.job_id,
        url: page.url,
        title: page.title,
        meta_description: page.meta_description,
        meta_keywords: None,
        canonical_url: page.canonical_url,
        word_count: page.word_count.unwrap_or(0),
        image_count: images.len() as i64,
        images_without_alt: count_images_without_alt(&images),
        internal_links,
        external_links,
        load_time,
        status_code: page.status_code,
        content_size: page.response_size_bytes.unwrap_or(0),
        mobile_friendly,
        has_structured_data,
        lighthouse_performance: lh_data.and_then(|lh| lh.performance_score),
        lighthouse_accessibility: lh_data.and_then(|lh| lh.accessibility_score),
        lighthouse_best_practices: lh_data.and_then(|lh| lh.best_practices_score),
        lighthouse_seo: lh_data.and_then(|lh| lh.seo_score),
        lighthouse_seo_audits,
        lighthouse_performance_metrics,
        images,
        detailed_links,
        headings,
        extracted_data,
    }
}

fn group_by<T, U, K>(
    items: Vec<T>,
    key: impl Fn(&T) -> K,
    convert: impl Fn(T) -> U,
) -> HashMap<K, Vec<U>>
where
    K: std::hash::Hash + Eq,
{
    let mut map: HashMap<K, Vec<U>> = HashMap::new();
    for item in items {
        let k = key(&item);
        map.entry(k).or_default().push(convert(item));
    }
    map
}

impl From<CompleteJobResult> for CompleteAnalysisResponse {
    fn from(result: CompleteJobResult) -> Self {
        let job = result.job;
        let pages = result.pages;
        let issues = result.issues;
        let links = result.links;
        let lighthouse = result.lighthouse;
        let headings = result.headings;
        let images = result.images;

        let page_url_by_id: HashMap<&str, &str> = pages
            .iter()
            .map(|p| (p.id.as_str(), p.url.as_str()))
            .collect();

        let assembled_issues: Vec<SeoIssue> = issues
            .into_iter()
            .map(|issue| {
                let page_id = issue.page_id.unwrap_or_default();
                let page_url = page_url_by_id
                    .get(page_id.as_str())
                    .map(|s| (*s).to_string())
                    .unwrap_or_default();
                SeoIssue {
                    page_url,
                    page_id,
                    severity: issue.severity,
                    title: issue.issue_type,
                    description: issue.message,
                    element: issue.details.clone(),
                    recommendation: issue.details.unwrap_or_default(),
                    line_number: None,
                }
            })
            .collect();

        let mut links_by_page = group_by(links, |l| l.source_page_id.clone(), LinkDetail::from);
        let mut headings_by_page = group_by(headings, |h| h.page_id.clone(), HeadingElement::from);
        let mut images_by_page = group_by(images, |i| i.page_id.clone(), ImageElement::from);

        let lighthouse_by_page: HashMap<String, LighthouseData> = lighthouse
            .into_iter()
            .map(|l| (l.page_id.clone(), l))
            .collect();

        let assembled_pages: Vec<PageAnalysisData> = pages
            .into_iter()
            .map(|mut p| {
                let page_id = p.id.clone();
                let extracted_data = std::mem::take(&mut p.extracted_data);
                assemble_page(
                    p,
                    lighthouse_by_page.get(&page_id),
                    links_by_page.remove(&page_id).unwrap_or_default(),
                    headings_by_page.remove(&page_id).unwrap_or_default(),
                    images_by_page.remove(&page_id).unwrap_or_default(),
                    extracted_data,
                )
            })
            .collect();

        let analysis = AnalysisResults {
            id: job.id.as_str().to_string(),
            url: job.url.clone(),
            status: job.status.clone(),
            progress: job.progress,
            total_pages: job.summary.total_pages(),
            analyzed_pages: job.summary.pages_crawled(),
            started_at: Some(job.created_at.to_rfc3339()),
            completed_at: job.completed_at.map(|d| d.to_rfc3339()),
            sitemap_found: job.sitemap_found,
            robots_txt_found: job.robots_txt_found,
            ssl_certificate: job.url.starts_with("https"),
            created_at: job.created_at.to_rfc3339(),
        };

        CompleteAnalysisResponse {
            summary: AnalysisSummary::compute(&job, &assembled_pages),
            analysis,
            pages: assembled_pages,
            issues: assembled_issues,
        }
    }
}
