use serde::{Deserialize, Serialize};
use specta::Type;

/// Quality / hardware tier for a curated model.
///
/// Replaces the previous `tier: String` field. The enum has a stable
/// `serde(rename_all = "lowercase")` representation, so the wire format
/// (`"small"` | `"medium"` | `"large"`) is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    Small,
    Medium,
    Large,
}

impl ModelTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

crate::impl_display_via_as_str!(ModelTier);

/// A model available for download from the curated registry.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tier: ModelTier,
    pub size_bytes: u64,
    pub download_url: String,
    pub filename: String,
    pub sha256: String,
}

/// Runtime state of a model: registry metadata + whether it's on disk.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModelInfo {
    #[serde(flatten)]
    pub entry: ModelEntry,
    pub is_downloaded: bool,
    pub is_active: bool,
    /// `true` when a partial `.tmp` file exists — the download can be resumed.
    pub has_partial: bool,
}

/// Curated model registry — three tiers, all public GGUF files from Hugging Face.
pub static MODEL_REGISTRY: std::sync::LazyLock<Vec<ModelEntry>> =
    std::sync::LazyLock::new(|| {
        vec![
            ModelEntry {
                id: "qwen2.5-7b-instruct-q4".to_string(),
                name: "Qwen 2.5 7B Instruct".to_string(),
                description: "Best quality. Strong reasoning and instruction following. Recommended for high-end machines.".to_string(),
                tier: ModelTier::Large,
                size_bytes: 4_680_000_000,
                download_url: "https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF/resolve/main/Qwen2.5-7B-Instruct-Q4_K_M.gguf".to_string(),
                filename: "Qwen2.5-7B-Instruct-Q4_K_M.gguf".to_string(),
                sha256: "65b8fcd92af6b4fefa935c625d1ac27ea29dcb6ee14589c55a8f115ceaaa1423".to_string(),
            },
            ModelEntry {
                id: "phi-4-mini-instruct-q4".to_string(),
                name: "Phi-4 Mini Instruct".to_string(),
                description: "Good balance of quality and speed. Microsoft's compact 3.8B model. Good for most machines.".to_string(),
                tier: ModelTier::Medium,
                size_bytes: 2_490_000_000,
                download_url: "https://huggingface.co/bartowski/microsoft_Phi-4-mini-instruct-GGUF/resolve/main/microsoft_Phi-4-mini-instruct-Q4_K_M.gguf".to_string(),
                filename: "microsoft_Phi-4-mini-instruct-Q4_K_M.gguf".to_string(),
                sha256: String::new(),
            },
            ModelEntry {
                id: "llama-3.2-1b-instruct-q4".to_string(),
                name: "Llama 3.2 1B Instruct".to_string(),
                description: "Fastest and smallest. Works on any machine. Lower quality but near-instant responses.".to_string(),
                tier: ModelTier::Small,
                size_bytes: 808_000_000,
                download_url: "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string(),
                filename: "Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string(),
                sha256: String::new(),
            },
        ]
    });

impl ModelEntry {
    pub fn find_by_id(id: &str) -> Option<&'static ModelEntry> {
        MODEL_REGISTRY.iter().find(|m| m.id == id)
    }
}

#[cfg(test)]
mod tests {
    //! Characterization tests for `ModelTier` (which replaced the old
    //! `tier: String` field) and the curated `MODEL_REGISTRY`. Pinning
    //! the registry's identity (3 entries, distinct ids, distinct tiers)
    //! and the wire format that the Tauri bindings depend on.

    use super::*;

    // ── ModelTier ────────────────────────────────────────────────────────

    #[test]
    fn model_tier_as_str_lowercase() {
        assert_eq!(ModelTier::Small.as_str(), "small");
        assert_eq!(ModelTier::Medium.as_str(), "medium");
        assert_eq!(ModelTier::Large.as_str(), "large");
    }

    #[test]
    fn model_tier_display_matches_as_str() {
        assert_eq!(format!("{}", ModelTier::Small), "small");
        assert_eq!(format!("{}", ModelTier::Medium), "medium");
        assert_eq!(format!("{}", ModelTier::Large), "large");
    }

    #[test]
    fn model_tier_serde_uses_lowercase_wire_format() {
        // The frontend expects the strings "small" / "medium" / "large".
        // Pinning the wire format (rename_all = "lowercase").
        assert_eq!(
            serde_json::to_string(&ModelTier::Small).unwrap(),
            "\"small\""
        );
        assert_eq!(
            serde_json::to_string(&ModelTier::Medium).unwrap(),
            "\"medium\""
        );
        assert_eq!(
            serde_json::to_string(&ModelTier::Large).unwrap(),
            "\"large\""
        );
    }

    #[test]
    fn model_tier_deserialize_from_lowercase() {
        let parsed: ModelTier = serde_json::from_str("\"small\"").unwrap();
        assert_eq!(parsed, ModelTier::Small);
        let parsed: ModelTier = serde_json::from_str("\"large\"").unwrap();
        assert_eq!(parsed, ModelTier::Large);
    }

    #[test]
    fn model_tier_is_copy_and_eq() {
        // Compile-time evidence + runtime equality.
        let a = ModelTier::Medium;
        let b = a; // Copy
        assert_eq!(a, b);
    }

    // ── MODEL_REGISTRY ───────────────────────────────────────────────────

    #[test]
    fn registry_contains_three_curated_entries() {
        assert_eq!(MODEL_REGISTRY.len(), 3);
    }

    #[test]
    fn registry_has_one_entry_per_tier() {
        let mut tiers: Vec<ModelTier> = MODEL_REGISTRY.iter().map(|m| m.tier).collect();
        tiers.sort_by_key(|t| t.as_str());
        assert_eq!(tiers, vec![ModelTier::Large, ModelTier::Medium, ModelTier::Small]);
    }

    #[test]
    fn registry_ids_are_unique() {
        let mut ids: Vec<&str> = MODEL_REGISTRY.iter().map(|m| m.id.as_str()).collect();
        ids.sort();
        let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, ids.len(), "duplicate id in MODEL_REGISTRY");
    }

    #[test]
    fn registry_filenames_are_unique() {
        let mut files: Vec<&str> = MODEL_REGISTRY.iter().map(|m| m.filename.as_str()).collect();
        files.sort();
        let unique_count = files
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(unique_count, files.len());
    }

    #[test]
    fn registry_size_bytes_are_positive_and_reasonable() {
        // Pin that no entry is mis-encoded as 0 or negative-looking via
        // overflow. Also pin reasonable bounds (100MB - 50GB).
        for entry in MODEL_REGISTRY.iter() {
            assert!(entry.size_bytes > 100_000_000, "{} too small", entry.id);
            assert!(entry.size_bytes < 50_000_000_000, "{} too big", entry.id);
        }
    }

    #[test]
    fn registry_download_urls_are_https() {
        for entry in MODEL_REGISTRY.iter() {
            assert!(
                entry.download_url.starts_with("https://"),
                "{} download URL not https",
                entry.id
            );
        }
    }

    // ── ModelEntry::find_by_id ───────────────────────────────────────────

    #[test]
    fn find_by_id_returns_some_for_known_ids() {
        for entry in MODEL_REGISTRY.iter() {
            let found = ModelEntry::find_by_id(&entry.id);
            assert!(found.is_some());
            assert_eq!(found.unwrap().id, entry.id);
        }
    }

    #[test]
    fn find_by_id_returns_none_for_unknown() {
        assert!(ModelEntry::find_by_id("not-a-real-model").is_none());
        assert!(ModelEntry::find_by_id("").is_none());
    }

    #[test]
    fn find_by_id_returns_static_reference() {
        // Compile-time evidence: the return type is &'static ModelEntry,
        // so the borrow can outlive any local scope.
        fn _check_lifetime() -> Option<&'static ModelEntry> {
            ModelEntry::find_by_id("phi-4-mini-instruct-q4")
        }
    }
}
