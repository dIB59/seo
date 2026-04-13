//! Small crate-wide helper macros. Kept separate from `html_selector.rs`
//! because these are not HTML-specific.

/// Derive a `std::fmt::Display` impl that forwards to a zero-arg
/// `as_str(&self) -> &'static str` method on the type. Replaces a dozen
/// near-identical one-line `impl Display` blocks across the domain
/// layer (`JobStatus`, `PageQueueStatus`, `PatternCategory`,
/// `PatternSeverity`, `BusinessImpact`, `FixEffort`).
///
/// Usage: `impl_display_via_as_str!(JobStatus);`
#[macro_export]
macro_rules! impl_display_via_as_str {
    ($ty:ty) => {
        impl ::std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}
