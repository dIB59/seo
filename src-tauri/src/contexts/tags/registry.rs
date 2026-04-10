//! Tag registry — runtime catalog assembled from the built-in field
//! list + live custom extractors.

use crate::contexts::extension::CustomExtractor;
use crate::repository::ExtensionRepository;

use super::model::{Tag, TagDataType, TagScope, TagSource};

/// The full tag catalog. Built on-demand from application state and
/// returned to the frontend via the `list_tags` command.
pub struct TagRegistry {
    tags: Vec<Tag>,
}

impl TagRegistry {
    /// Build the registry from live application state. One
    /// `list_extractors` round-trip plus an in-memory extend.
    pub async fn build(
        extension_repo: &dyn ExtensionRepository,
    ) -> crate::repository::RepositoryResult<Self> {
        let extractors = extension_repo.list_extractors().await?;
        Ok(Self::from_parts(builtin_tags(), extractors))
    }

    /// Construct directly from pre-fetched data. Used by `build` and
    /// exposed for unit tests that don't need an async repo.
    pub fn from_parts(builtins: Vec<Tag>, extractors: Vec<CustomExtractor>) -> Self {
        let mut tags = builtins;
        tags.extend(extractors.into_iter().map(extractor_to_tag));
        Self { tags }
    }

    /// Look up a tag by its `name` (exact match, case-sensitive).
    pub fn lookup(&self, name: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.name == name)
    }

    /// Return only tags that are valid in the given scope.
    pub fn in_scope(&self, scope: TagScope) -> Vec<&Tag> {
        self.tags.iter().filter(|t| t.scopes.contains(&scope)).collect()
    }

    /// Consume the registry into its inner tag list (for
    /// serialization / Tauri command return).
    pub fn into_tags(self) -> Vec<Tag> {
        self.tags
    }
}

// ── Extractor → Tag projection ───────────────────────────────────────────────

fn extractor_to_tag(ext: CustomExtractor) -> Tag {
    let data_type = if ext.multiple {
        TagDataType::List
    } else {
        TagDataType::Text
    };

    Tag {
        name: format!("tag:{}", ext.tag),
        label: ext.name.clone(),
        description: format!(
            "Custom extractor — CSS: {} {}",
            ext.selector,
            ext.attribute
                .as_deref()
                .map(|a| format!("@{a}"))
                .unwrap_or_else(|| "text content".into())
        ),
        data_type,
        source: TagSource::Extractor {
            extractor_id: ext.id,
            extractor_name: ext.name,
        },
        scopes: vec![
            TagScope::CheckField,
            TagScope::CheckMessage,
            TagScope::TemplateText,
            TagScope::AiPrompt,
        ],
        example: None,
    }
}

// ── Built-in catalog ─────────────────────────────────────────────────────────
//
// Two groups: site-level variables (used in templates / prompts) and
// page-level fields (used in check rules / message templates). The
// division matches the scopes they carry.

fn builtin_tags() -> Vec<Tag> {
    let mut tags = Vec::with_capacity(24);
    tags.extend(site_level_tags());
    tags.extend(page_level_tags());
    tags
}

/// Exposed for unit tests so they can call `TagRegistry::from_parts`
/// with the canonical built-in list without going through `build`.
#[doc(hidden)]
pub fn __test_support_builtins() -> Vec<Tag> {
    builtin_tags()
}

fn site_level_tags() -> Vec<Tag> {
    let tpl = vec![TagScope::TemplateText, TagScope::AiPrompt];
    let tpl_cond = vec![
        TagScope::TemplateText,
        TagScope::AiPrompt,
        TagScope::TemplateCondition,
    ];

    vec![
        Tag {
            name: "url".into(),
            label: "Site URL".into(),
            description: "The base URL of the site being audited.".into(),
            data_type: TagDataType::Text,
            source: TagSource::Builtin,
            scopes: tpl.clone(),
            example: Some("https://example.com".into()),
        },
        Tag {
            name: "score".into(),
            label: "SEO Score".into(),
            description: "Overall SEO score (0–100).".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("87".into()),
        },
        Tag {
            name: "pages_count".into(),
            label: "Pages Analyzed".into(),
            description: "Number of pages in this audit.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl.clone(),
            example: Some("42".into()),
        },
        Tag {
            name: "total_issues".into(),
            label: "Total Issues".into(),
            description: "Total issue count across all severities.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("17".into()),
        },
        Tag {
            name: "critical_issues".into(),
            label: "Critical Issues".into(),
            description: "Number of critical-severity issues.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("3".into()),
        },
        Tag {
            name: "warning_issues".into(),
            label: "Warnings".into(),
            description: "Number of warning-severity issues.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("9".into()),
        },
        Tag {
            name: "sitemap_found".into(),
            label: "Sitemap Found".into(),
            description: "Whether a sitemap.xml was found.".into(),
            data_type: TagDataType::Text,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("Yes".into()),
        },
        Tag {
            name: "robots_txt_found".into(),
            label: "Robots.txt Found".into(),
            description: "Whether a robots.txt was found.".into(),
            data_type: TagDataType::Text,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("No".into()),
        },
        Tag {
            name: "pillar.technical".into(),
            label: "Technical Pillar".into(),
            description: "Technical pillar score (0–100).".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("78".into()),
        },
        Tag {
            name: "pillar.content".into(),
            label: "Content Pillar".into(),
            description: "Content pillar score (0–100).".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("62".into()),
        },
        Tag {
            name: "pillar.performance".into(),
            label: "Performance Pillar".into(),
            description: "Performance pillar score (0–100).".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("91".into()),
        },
        Tag {
            name: "pillar.accessibility".into(),
            label: "Accessibility Pillar".into(),
            description: "Accessibility pillar score (0–100).".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond.clone(),
            example: Some("85".into()),
        },
        Tag {
            name: "pillar.overall".into(),
            label: "Overall Pillar Average".into(),
            description: "Arithmetic mean of the four pillar scores.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: tpl_cond,
            example: Some("79".into()),
        },
    ]
}

fn page_level_tags() -> Vec<Tag> {
    let check = vec![TagScope::CheckField, TagScope::CheckMessage];

    vec![
        Tag {
            name: "title".into(),
            label: "Page Title".into(),
            description: "The <title> tag text.".into(),
            data_type: TagDataType::Text,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("My Page Title".into()),
        },
        Tag {
            name: "meta_description".into(),
            label: "Meta Description".into(),
            description: "The meta description content.".into(),
            data_type: TagDataType::Text,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("A concise page summary.".into()),
        },
        Tag {
            name: "canonical_url".into(),
            label: "Canonical URL".into(),
            description: "The canonical link href, if present.".into(),
            data_type: TagDataType::Text,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("https://example.com/page".into()),
        },
        Tag {
            name: "word_count".into(),
            label: "Word Count".into(),
            description: "Number of words in the page body.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("1200".into()),
        },
        Tag {
            name: "load_time_ms".into(),
            label: "Load Time (ms)".into(),
            description: "Page load time in milliseconds.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("850".into()),
        },
        Tag {
            name: "status_code".into(),
            label: "HTTP Status Code".into(),
            description: "The HTTP status code returned for this page.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("200".into()),
        },
        Tag {
            name: "has_viewport".into(),
            label: "Has Viewport".into(),
            description: "Whether the page has a viewport meta tag.".into(),
            data_type: TagDataType::Bool,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("true".into()),
        },
        Tag {
            name: "has_structured_data".into(),
            label: "Has Structured Data".into(),
            description: "Whether JSON-LD structured data is present.".into(),
            data_type: TagDataType::Bool,
            source: TagSource::Builtin,
            scopes: check.clone(),
            example: Some("false".into()),
        },
        Tag {
            name: "h1_count".into(),
            label: "H1 Count".into(),
            description: "Number of <h1> headings on the page.".into(),
            data_type: TagDataType::Number,
            source: TagSource::Builtin,
            scopes: check,
            example: Some("1".into()),
        },
    ]
}
