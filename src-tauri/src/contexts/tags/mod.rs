//! Tags — the user-facing symbol catalog.
//!
//! A [`Tag`] is any named symbol a consultant can reference from their
//! customization surface (custom checks, report templates, AI prompts).
//! Every tag has a source (where it comes from), a data type, a
//! description, and a set of scopes it's valid in.
//!
//! The registry is **computed at runtime** from live application state
//! — extractors from the repository, plus a small built-in catalog of
//! page fields. Not persisted. This keeps the "what tags exist"
//! question always-correct: adding a custom extractor immediately adds
//! a new tag, and deleting one immediately removes it.
//!
//! ## Design notes (chunk 2 of the tags feature)
//!
//! - `TagSource` is an enum with room to grow. Today it has
//!   `Extractor` and `Builtin`; future chunks may add `Pattern`,
//!   `IssueType`, etc. Call sites that want "tags of kind X" use
//!   `TagSource` match arms.
//! - The registry is cheap to rebuild — a single `list_extractors`
//!   round trip plus an in-memory extend of a small constant vec —
//!   so consumers just call `TagRegistry::build` on demand rather
//!   than caching with invalidation.
//! - `Tag.name` is **exactly** the string the user types when
//!   referencing it: `"url"`, `"tag:og_image"`, `"critical_issues"`.
//!   Not a template placeholder (no braces) — placeholders are an
//!   orthogonal concern handled by the substitution engine.

mod model;
mod registry;

pub use model::{Tag, TagDataType, TagScope, TagSource};
pub use registry::TagRegistry;

#[cfg(test)]
mod tests;
