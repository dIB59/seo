//! Tag data model — the wire format returned by `list_tags` and
//! consumed by the template / check editors.

use serde::{Deserialize, Serialize};
use specta::Type;

/// A named symbol the consultant can reference when authoring a custom
/// check, a report template, or an AI prompt. `name` is exactly what
/// the user types.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// The literal string the user references this tag with. For
    /// built-in page fields this is the bare field name (`"title"`,
    /// `"word_count"`). For custom-extractor tags it's the prefixed
    /// form (`"tag:og_image"`) that can be dropped directly into a
    /// check `field` dropdown.
    pub name: String,

    /// Human-readable label for the editor UI.
    pub label: String,

    /// One-line description shown next to the tag in the picker.
    pub description: String,

    /// The data type the tag resolves to. Drives operator
    /// compatibility in the custom-check editor — e.g. an operator
    /// like `Lt` only makes sense on `Number` tags.
    pub data_type: TagDataType,

    /// Where the tag comes from. The editor uses this to group tags
    /// visually ("Built-in" / "Your Extractors" / etc.).
    pub source: TagSource,

    /// Which authoring surfaces this tag is valid in. A tag may appear
    /// in more than one — e.g. `{critical_issues}` is valid in both a
    /// template text block and a template conditional.
    pub scopes: Vec<TagScope>,

    /// An example value the editor can display in a tooltip to show
    /// what the tag resolves to in practice. `None` means "no
    /// representative example available".
    pub example: Option<String>,
}

/// Where a tag originates. The enum is deliberately open-ended — future
/// chunks add new variants (pattern IDs, issue-type literals, client
/// branding fields) without breaking existing call sites.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TagSource {
    /// Hardcoded page or site field known to the analyzer — e.g.
    /// `title`, `word_count`, `critical_issues`.
    Builtin,

    /// A user-defined custom extractor. `extractor_id` is the UUID of
    /// the `custom_extractors` row; `extractor_name` is the
    /// human-readable name the user gave it.
    Extractor {
        extractor_id: String,
        extractor_name: String,
    },
}

/// The underlying data type a tag resolves to. Used by the editor to
/// suggest compatible operators and to format the example value.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TagDataType {
    /// A free-form text value. Compatible with `eq`, `contains`,
    /// `not_contains`, `present`, `missing`.
    Text,
    /// An integer or float. Compatible with `eq`, `lt`, `gt`,
    /// `present`, `missing`.
    Number,
    /// A true/false flag. Compatible with `eq`, `present`, `missing`.
    Bool,
    /// A list of values (e.g. hreflang returns multiple codes).
    /// Compatible with `present`, `missing`, `contains`.
    List,
}

/// An authoring surface where a tag can be referenced. The editor
/// filters the tag picker to the scopes that match the current field.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum TagScope {
    /// The `field` dropdown on a custom check or report pattern.
    /// Only tags that can be *evaluated* against a page belong here.
    CheckField,
    /// The `message_template` on a custom check — string substitution
    /// into an issue message. Resolves per-page when the check fires.
    CheckMessage,
    /// A text block inside a report template. Resolves at render
    /// time, site-level scope.
    TemplateText,
    /// A conditional wrapper inside a report template. Only tags that
    /// carry boolean or numeric semantics belong here.
    TemplateCondition,
    /// An AI prompt block. Same resolution semantics as
    /// `TemplateText`.
    AiPrompt,
}
