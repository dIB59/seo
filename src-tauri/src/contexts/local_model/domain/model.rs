use serde::{Deserialize, Serialize};
use specta::Type;

/// A model available for download from the curated registry.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    /// "small" | "medium" | "large"
    pub tier: String,
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
}

/// Curated model registry — three tiers, all public GGUF files from Hugging Face.
pub static MODEL_REGISTRY: std::sync::LazyLock<Vec<ModelEntry>> =
    std::sync::LazyLock::new(|| {
        vec![
            ModelEntry {
                id: "qwen2.5-7b-instruct-q4".to_string(),
                name: "Qwen 2.5 7B Instruct".to_string(),
                description: "Best quality. Strong reasoning and instruction following. Recommended for high-end machines.".to_string(),
                tier: "large".to_string(),
                size_bytes: 4_680_000_000,
                download_url: "https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF/resolve/main/Qwen2.5-7B-Instruct-Q4_K_M.gguf".to_string(),
                filename: "Qwen2.5-7B-Instruct-Q4_K_M.gguf".to_string(),
                sha256: "65b8fcd92af6b4fefa935c625d1ac27ea29dcb6ee14589c55a8f115ceaaa1423".to_string(),
            },
            ModelEntry {
                id: "phi-4-mini-instruct-q4".to_string(),
                name: "Phi-4 Mini Instruct".to_string(),
                description: "Good balance of quality and speed. Microsoft's compact 3.8B model. Good for most machines.".to_string(),
                tier: "medium".to_string(),
                size_bytes: 2_490_000_000,
                download_url: "https://huggingface.co/bartowski/microsoft_Phi-4-mini-instruct-GGUF/resolve/main/microsoft_Phi-4-mini-instruct-Q4_K_M.gguf".to_string(),
                filename: "microsoft_Phi-4-mini-instruct-Q4_K_M.gguf".to_string(),
                sha256: String::new(),
            },
            ModelEntry {
                id: "llama-3.2-1b-instruct-q4".to_string(),
                name: "Llama 3.2 1B Instruct".to_string(),
                description: "Fastest and smallest. Works on any machine. Lower quality but near-instant responses.".to_string(),
                tier: "small".to_string(),
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
