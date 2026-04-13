use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::Result;
use async_trait::async_trait;

pub struct InferenceRequest {
    pub model_path: PathBuf,
    pub prompt: String,
    pub max_tokens: usize,
    pub temperature: f32,
}

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn infer(&self, request: InferenceRequest) -> Result<String>;
}

/// Llama.cpp inference engine backed by the `llama-cpp-2` crate.
pub struct LlamaInferenceEngine;

impl LlamaInferenceEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LlamaInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InferenceEngine for LlamaInferenceEngine {
    async fn infer(&self, request: InferenceRequest) -> Result<String> {
        tokio::task::spawn_blocking(move || run_inference(request))
            .await
            .map_err(|e| anyhow::anyhow!("Inference task panicked: {e}"))?
    }
}

// ── Global backend (llama.cpp can only be initialized once per process) ──────
//
// We store `Result<LlamaBackend, String>` so that initialization errors are
// surfaced as `Err(...)` on every call instead of panicking.

type BackendResult = std::result::Result<llama_cpp_2::llama_backend::LlamaBackend, String>;
static LLAMA_BACKEND: OnceLock<BackendResult> = OnceLock::new();

fn backend() -> Result<&'static llama_cpp_2::llama_backend::LlamaBackend> {
    LLAMA_BACKEND
        .get_or_init(|| {
            llama_cpp_2::llama_backend::LlamaBackend::init()
                .map_err(|e| format!("Failed to initialize llama backend: {e}"))
        })
        .as_ref()
        .map_err(|e| anyhow::anyhow!("{e}"))
}

// ── Synchronous inference (runs on a blocking thread) ────────────────────────

fn run_inference(req: InferenceRequest) -> Result<String> {
    tracing::info!(
        "[LLAMA] inference start — model: {}, prompt len: {} chars, max_tokens: {}, temp: {:.2}",
        req.model_path.display(),
        req.prompt.len(),
        req.max_tokens,
        req.temperature,
    );
    tracing::debug!(
        "[LLAMA] prompt (first 200 chars):\n{}{}",
        &req.prompt[..req.prompt.len().min(200)],
        if req.prompt.len() > 200 { " […]" } else { "" }
    );
    use llama_cpp_2::context::params::LlamaContextParams;
    use llama_cpp_2::llama_batch::LlamaBatch;
    use llama_cpp_2::model::params::LlamaModelParams;
    use llama_cpp_2::model::{AddBos, LlamaModel};
    use llama_cpp_2::sampling::LlamaSampler;

    let backend = backend()?;

    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(backend, &req.model_path, &model_params)
        .map_err(|e| anyhow::anyhow!("Failed to load model: {e}"))?;

    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(2048));
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| anyhow::anyhow!("Failed to create context: {e}"))?;

    // Tokenize the prompt
    let tokens = model
        .str_to_token(&req.prompt, AddBos::Always)
        .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

    let n_prompt = tokens.len();
    if n_prompt == 0 {
        return Ok(String::new());
    }

    // Fill prompt batch (logits only for the last token)
    let mut batch = LlamaBatch::new(n_prompt.max(1), 1);
    for (i, &token) in tokens.iter().enumerate() {
        let is_last = i == n_prompt - 1;
        batch
            .add(token, i as i32, &[0], is_last)
            .map_err(|e| anyhow::anyhow!("Batch add error: {e}"))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| anyhow::anyhow!("Prompt decode failed: {e}"))?;

    // Sampler: repetition penalty → temperature → greedy.
    // The penalty looks back 64 tokens and penalizes repeats — this
    // prevents the "To ensure continuous progress…" infinite loop that
    // small models fall into when the prompt is long relative to the
    // context window.
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::penalties(
            64,   // penalty_last_n: look back 64 tokens
            1.15, // penalty_repeat: >1.0 penalizes repetition
            0.0,  // penalty_freq: frequency penalty (off)
            0.0,  // penalty_present: presence penalty (off)
        ),
        LlamaSampler::temp(req.temperature),
        LlamaSampler::greedy(),
    ]);

    // Generation loop — one token at a time after the prompt
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut output = String::new();
    let mut n_pos = n_prompt as i32;

    for _ in 0..req.max_tokens {
        let token = sampler.sample(&ctx, -1);

        if model.is_eog_token(token) {
            break;
        }

        let piece = model
            .token_to_piece(token, &mut decoder, false, None)
            .map_err(|e| anyhow::anyhow!("Token decode error: {e}"))?;
        output.push_str(&piece);

        // Early exit: if the last 200 chars repeat a phrase 3+ times,
        // the model is stuck. Truncate to just before the first repeat.
        if output.len() > 200 {
            if let Some(clean) = detect_and_truncate_repetition(&output) {
                tracing::warn!(
                    "[LLAMA] repetition detected at {} chars, truncating to {}",
                    output.len(),
                    clean.len()
                );
                output = clean;
                break;
            }
        }

        batch.clear();
        batch
            .add(token, n_pos, &[0], true)
            .map_err(|e| anyhow::anyhow!("Batch add error: {e}"))?;
        ctx.decode(&mut batch)
            .map_err(|e| anyhow::anyhow!("Token decode failed: {e}"))?;
        n_pos += 1;
    }

    tracing::info!("[LLAMA] inference done — output len: {} chars", output.len());
    tracing::debug!(
        "[LLAMA] output (first 200 chars):\n{}{}",
        &output[..output.len().min(200)],
        if output.len() > 200 { " […]" } else { "" }
    );

    Ok(output.trim().to_string())
}

/// Detect if the output has fallen into a repetition loop. Scans the
/// last portion of the text for any sentence-length substring (20+
/// chars) that appears 3 or more times. Returns the text truncated to
/// just before the second occurrence, or `None` if no repetition found.
fn detect_and_truncate_repetition(text: &str) -> Option<String> {
    // Only check the tail — early text is fine
    let check_len = text.len().min(800);
    let tail = &text[text.len() - check_len..];

    // Try to find a repeated sentence by looking for repeated chunks.
    // Slide a window of 30..80 chars and see if it appears 3+ times
    // in the tail.
    for window_size in [60, 50, 40, 30] {
        if tail.len() < window_size * 3 {
            continue;
        }
        // Take a chunk from the end and count occurrences
        let chunk = &tail[tail.len() - window_size..];
        let count = tail.matches(chunk).count();
        if count >= 3 {
            // Find the first occurrence in the full text and keep up to
            // the second occurrence
            if let Some(first) = text.find(chunk) {
                let after_first = first + chunk.len();
                if let Some(second) = text[after_first..].find(chunk) {
                    let truncate_at = after_first + second;
                    return Some(text[..truncate_at].trim().to_string());
                }
            }
        }
    }
    None
}
