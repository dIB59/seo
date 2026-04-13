//! Data model for report templates.
//!
//! These types are the wire format stored in the `report_templates`
//! table (chunk 3) and sent to the frontend template editor (chunk 6).
//! Every type derives `serde::{Serialize, Deserialize}` + `specta::Type`
//! so the same struct serves as the on-disk JSON, the HTTP-ish Tauri
//! boundary, and the internal Rust domain.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::condition::{Condition, PatternFilter};

/// A named, reorderable report template authored by the consultant.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    /// Builtin templates (the "Default" one seeded at migration time)
    /// cannot be deleted, only disabled. Mirrors `ReportPattern.is_builtin`.
    pub is_builtin: bool,
    pub sections: Vec<TemplateSection>,
    /// Which custom extractor tags this template includes in `{tag_summary}`
    /// and makes available as `{tag.X}` variables. The user picks these in
    /// the template editor from the live tag registry.
    ///
    /// Empty list = include ALL tags (backwards compat + sensible default).
    /// Non-empty = only these tag names (e.g. `["og_image", "author"]`).
    #[serde(default)]
    pub selected_tags: Vec<String>,
}

/// One section of a report. Each variant is self-contained — the render
/// pipeline walks the section list and produces fragments independently
/// per section, so adding a new section kind is purely additive.
///
/// Serde uses an internally-tagged `{ "kind": "text", ... }` representation
/// so the JSON on disk stays readable.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TemplateSection {
    /// A markdown heading. `level` is 1..=6.
    Heading {
        level: u8,
        /// Heading text, may contain `{variable}` placeholders.
        text: String,
    },

    /// Static prose with variable substitution.
    ///
    /// The template is a single string which may contain `{url}`,
    /// `{score}`, `{critical_issues}`, etc. — anything supported by
    /// `service::prompt::replace_prompt_vars` plus the new pattern
    /// variables (`{top_patterns}`, `{detected_patterns_count}`).
    Text { template: String },

    /// An LLM prompt the renderer should expand asynchronously.
    ///
    /// Chunk 1 just emits an `AiPrompt(String)` fragment; chunk 7 wires
    /// it to the actual AI backends. The prompt is variable-substituted
    /// before being handed to the LLM, so consultants can parameterize
    /// their prompts per audit.
    Ai {
        /// Human-facing label shown in the template editor. Not sent to
        /// the LLM.
        label: String,
        /// The prompt text, with variable placeholders.
        prompt: String,
    },

    /// "For each detected pattern matching `filter`, render
    /// `per_pattern_template` with that pattern's fields in scope."
    ///
    /// Per-pattern substitution adds `{pattern.name}`, `{pattern.pct}`,
    /// `{pattern.affected_pages}`, `{pattern.recommendation}`,
    /// `{pattern.category}`, `{pattern.severity}` on top of the
    /// context-level variables.
    PatternSummary {
        filter: PatternFilter,
        per_pattern_template: String,
        /// Rendered in place of `per_pattern_template` when the filter
        /// matches zero patterns. `None` → nothing is emitted at all.
        empty_template: Option<String>,
    },

    /// Wrap a list of child sections in a runtime condition. Children
    /// are only rendered when the condition evaluates to true.
    Conditional {
        when: Condition,
        children: Vec<TemplateSection>,
    },

    /// Horizontal divider. Renders as a markdown `---`.
    Divider,
}

impl ReportTemplate {
    /// Construct an empty template with the given id and name. Useful
    /// for tests and the "new template" button in the editor.
    pub fn empty(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            is_builtin: false,
            sections: Vec::new(),
            selected_tags: Vec::new(),
        }
    }
}
