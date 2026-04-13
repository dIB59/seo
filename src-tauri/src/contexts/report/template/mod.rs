//! Report template engine — the customization surface for report output.
//!
//! A [`ReportTemplate`] is an ordered list of [`TemplateSection`]s. Each
//! section is one of: static text with variable substitution, an AI
//! prompt marker (executed later by the renderer's consumer), a pattern
//! summary, a conditional wrapper, a heading, or a divider.
//!
//! The engine is backend-only in this first chunk — it has zero storage
//! and zero wiring into `brief_builder`. It exists purely so subsequent
//! chunks have a target to land against.
//!
//! ## Why a template engine at all?
//!
//! Today the report brief is hardcoded in `brief_builder::phase1/2/3` —
//! which means "customization" for a consultant's client requires a
//! code change. Moving that structure into user-authored JSON lets
//! consultants ship their own voice per client without forking the
//! binary.
//!
//! ## What runs where
//!
//! - `render_template` is pure and synchronous. Given a template + a
//!   `RenderContext`, it walks the section list, resolves every variable
//!   against the context, evaluates every conditional, and emits a
//!   `Vec<RenderedFragment>`.
//! - `RenderedFragment` is either finished markdown text **or** an
//!   `AiPrompt` marker. AI markers are the only asynchronous bit — a
//!   separate consumer (chunk 7) will walk the fragments, send each
//!   `AiPrompt` through the configured backend, and stitch the results
//!   into the final string.
//! - Chunks 1–6 never hit an LLM. Tests are all synchronous and
//!   deterministic.

mod condition;
pub mod defaults;
mod engine;
mod model;

pub use condition::{Condition, PatternFilter};
pub use engine::{render_template, render_template_to_string, RenderContext, RenderError, RenderedFragment};
pub use model::{ReportTemplate, TemplateSection};

#[cfg(test)]
mod tests;
