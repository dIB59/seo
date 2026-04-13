//! Tag registry unit tests. These pin the built-in catalog and the
//! extractor projection so a future refactor can't silently drop a
//! field or mis-scope a tag.

use super::*;
use crate::contexts::extension::CustomExtractor;

fn single_extractor_text() -> CustomExtractor {
    CustomExtractor {
        id: "ext-1".into(),
        name: "OG Image".into(),
        tag: "og_image".into(),
        selector: "meta[property='og:image']".into(),
        attribute: Some("content".into()),
        multiple: false,
        enabled: true,
    }
}

fn single_extractor_multi() -> CustomExtractor {
    CustomExtractor {
        id: "ext-2".into(),
        name: "Hreflang".into(),
        tag: "hreflang".into(),
        selector: "link[rel='alternate'][hreflang]".into(),
        attribute: Some("hreflang".into()),
        multiple: true,
        enabled: true,
    }
}

#[test]
fn builtin_catalog_contains_the_site_level_variables() {
    // Every site-level variable the brief_builder / template engine
    // already uses MUST be in the registry, otherwise the editor
    // won't surface them.
    let registry = TagRegistry::from_parts(
        crate::contexts::tags::registry::__test_support_builtins(),
        vec![],
    );
    for expected in [
        "url",
        "score",
        "pages_count",
        "total_issues",
        "critical_issues",
        "warning_issues",
        "sitemap_found",
        "robots_txt_found",
        "pillar.technical",
        "pillar.content",
        "pillar.performance",
        "pillar.accessibility",
        "pillar.overall",
    ] {
        assert!(
            registry.lookup(expected).is_some(),
            "missing built-in tag: {expected}"
        );
    }
}

#[test]
fn builtin_catalog_contains_page_level_fields_used_by_resolve_field() {
    // The `pattern_engine::resolve_field` function matches on these
    // field names; the registry must offer them as CheckField tags.
    let registry = TagRegistry::from_parts(
        crate::contexts::tags::registry::__test_support_builtins(),
        vec![],
    );
    for expected in [
        "title",
        "meta_description",
        "canonical_url",
        "word_count",
        "load_time_ms",
        "status_code",
        "has_viewport",
        "has_structured_data",
        "h1_count",
    ] {
        let tag = registry
            .lookup(expected)
            .unwrap_or_else(|| panic!("missing page field tag: {expected}"));
        assert!(
            tag.scopes.contains(&TagScope::CheckField),
            "tag {expected} should be usable as a check field"
        );
    }
}

#[test]
fn extractor_projection_uses_tag_prefix() {
    let registry = TagRegistry::from_parts(
        vec![],
        vec![single_extractor_text()],
    );
    let tag = registry.lookup("tag:og_image").expect("extractor tag should exist");
    assert_eq!(tag.label, "OG Image");
    assert_eq!(tag.data_type, TagDataType::Text);
    match &tag.source {
        TagSource::Extractor { extractor_id, extractor_name } => {
            assert_eq!(extractor_id, "ext-1");
            assert_eq!(extractor_name, "OG Image");
        }
        other => panic!("expected TagSource::Extractor, got {other:?}"),
    }
}

#[test]
fn multi_valued_extractor_produces_list_typed_tag() {
    let registry = TagRegistry::from_parts(
        vec![],
        vec![single_extractor_multi()],
    );
    let tag = registry.lookup("tag:hreflang").expect("extractor tag should exist");
    assert_eq!(tag.data_type, TagDataType::List);
}

#[test]
fn in_scope_filters_to_matching_tags() {
    let registry = TagRegistry::from_parts(
        crate::contexts::tags::registry::__test_support_builtins(),
        vec![single_extractor_text()],
    );

    // Template text should include site-level vars + extractor tag,
    // but NOT page-level check fields like `word_count`.
    let template_tags: Vec<&str> = registry
        .in_scope(TagScope::TemplateText)
        .iter()
        .map(|t| t.name.as_str())
        .collect();

    assert!(template_tags.contains(&"url"));
    assert!(template_tags.contains(&"score"));
    assert!(template_tags.contains(&"tag:og_image"));
    // word_count is CheckField / CheckMessage only — not TemplateText.
    assert!(!template_tags.contains(&"word_count"));

    // CheckField should include the page fields and the extractor tag
    // but NOT the site-level narrative variables.
    let check_tags: Vec<&str> = registry
        .in_scope(TagScope::CheckField)
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert!(check_tags.contains(&"word_count"));
    assert!(check_tags.contains(&"tag:og_image"));
    assert!(!check_tags.contains(&"url"));
    assert!(!check_tags.contains(&"critical_issues"));
}

#[test]
fn lookup_returns_none_for_unknown_tag() {
    let registry = TagRegistry::from_parts(
        crate::contexts::tags::registry::__test_support_builtins(),
        vec![],
    );
    assert!(registry.lookup("nonsense").is_none());
    assert!(registry.lookup("tag:does_not_exist").is_none());
}

#[test]
fn into_tags_preserves_insertion_order_builtin_first_extractors_last() {
    // Pinning the visual order the editor gets: built-ins come first
    // (so the default catalog is stable), extractors come after in
    // creation order.
    let registry = TagRegistry::from_parts(
        crate::contexts::tags::registry::__test_support_builtins(),
        vec![single_extractor_text(), single_extractor_multi()],
    );
    let tags = registry.into_tags();
    let mut saw_extractor = false;
    for tag in &tags {
        if matches!(tag.source, TagSource::Extractor { .. }) {
            saw_extractor = true;
        } else {
            // A built-in must not appear after any extractor.
            assert!(
                !saw_extractor,
                "built-in tag {:?} appeared after an extractor tag — ordering is wrong",
                tag.name
            );
        }
    }
    // And the two extractors are in the order they were passed in.
    let extractor_names: Vec<&str> = tags
        .iter()
        .filter_map(|t| match &t.source {
            TagSource::Extractor { .. } => Some(t.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(extractor_names, vec!["tag:og_image", "tag:hreflang"]);
}
