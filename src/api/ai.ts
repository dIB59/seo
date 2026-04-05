import { toast } from "sonner";
import { Result } from "../lib/result";
import { commands, type CompleteAnalysisResponse, type SeoIssue } from "@/src/bindings";

export const AiError = {
  MissingKey: "MISSING_KEY",
  InvalidKey: "INVALID_KEY",
  RateLimit: "RATE_LIMIT",
  NetworkError: "NETWORK_ERROR",
  Unknown: "UNKNOWN",
} as const;

export type AiError = (typeof AiError)[keyof typeof AiError];

async function getStoredApiKey(): Promise<Result<string, string>> {
  const res = await commands.getGeminiApiKey();
  if (res.status === "ok") {
    const key = res.data;
    if (key && key.trim().length > 0) return Result.Ok(key);
    return Result.Err("API_KEY_MISSING");
  }

  return Result.Err(res.error ?? "API_KEY_MISSING");
}

function openSettingsDialog() {
  window.dispatchEvent(new CustomEvent("open-settings-dialog"));
}

function mapErrorToType(error: string): AiError {
  if (error.includes("API_KEY_MISSING")) return AiError.MissingKey;
  if (error.includes("401")) return AiError.InvalidKey;
  if (error.includes("429")) return AiError.RateLimit;
  return AiError.Unknown;
}

export async function generateGeminiAnalysis(
  result: CompleteAnalysisResponse,
): Promise<Result<string, AiError>> {
  const apiKeyResult = await getStoredApiKey();

  if (apiKeyResult.isErr() || !apiKeyResult.unwrap()) {
    handleAiUiEffects(AiError.MissingKey);
    return Result.Err(AiError.MissingKey);
  }

  const { analysis, summary, issues, pages } = result;

  const insightsResult = await commands.getGeminiInsights({
    analysis_id: analysis.id,
    url: analysis.url,
    seo_score: summary.seo_score,
    pages_count: pages.length,
    total_issues: summary.total_issues,
    critical_issues: issues.filter((i: SeoIssue) => i.severity === "critical").length,
    warning_issues: issues.filter((i: SeoIssue) => i.severity === "warning").length,
    suggestion_issues: issues.filter((i: SeoIssue) => i.severity === "info").length,
    top_issues: issues.slice(0, 10).map((i: SeoIssue) => `- ${i.title}`),
    avg_load_time: summary.avg_load_time,
    total_words: summary.total_words,
    ssl_certificate: analysis.ssl_certificate,
    sitemap_found: analysis.sitemap_found,
    robots_txt_found: analysis.robots_txt_found,
  });

  if (insightsResult.status === "ok") {
    return Result.Ok(insightsResult.data);
  }

  const errorType = mapErrorToType(insightsResult.error as string);
  handleAiUiEffects(errorType);
  return Result.Err(errorType);
}

export async function get_gemini_api_key(): Promise<Result<string | null, string>> {
  const res = await commands.getGeminiApiKey();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function set_gemini_api_key(apiKey: string): Promise<Result<null, string>> {
  const res = await commands.setGeminiApiKey(apiKey);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function get_gemini_persona(): Promise<Result<string | null, string>> {
  const res = await commands.getGeminiPersona();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function set_gemini_persona(persona: string): Promise<Result<null, string>> {
  const res = await commands.setGeminiPersona(persona);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function get_gemini_prompt_blocks(): Promise<Result<string | null, string>> {
  const res = await commands.getGeminiPromptBlocks();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function set_gemini_prompt_blocks(blocks: string): Promise<Result<null, string>> {
  const res = await commands.setGeminiPromptBlocks(blocks);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function get_gemini_enabled(): Promise<Result<boolean, string>> {
  const res = await commands.getGeminiEnabled();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function set_gemini_enabled(enabled: boolean): Promise<Result<null, string>> {
  const res = await commands.setGeminiEnabled(enabled);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function get_gemini_requirements(): Promise<Result<string | null, string>> {
  const res = await commands.getGeminiRequirements();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function set_gemini_requirements(requirements: string): Promise<Result<null, string>> {
  const res = await commands.setGeminiRequirements(requirements);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function get_gemini_context_options(): Promise<Result<string | null, string>> {
  const res = await commands.getGeminiContextOptions();
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

export async function set_gemini_context_options(options: string): Promise<Result<null, string>> {
  const res = await commands.setGeminiContextOptions(options);
  return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error ?? "");
}

function buildInsightsPayload(result: CompleteAnalysisResponse) {
  const { analysis, summary, issues, pages } = result;
  return {
    analysis_id: analysis.id,
    url: analysis.url,
    seo_score: summary.seo_score,
    pages_count: pages.length,
    total_issues: summary.total_issues,
    critical_issues: issues.filter((i: SeoIssue) => i.severity === "critical").length,
    warning_issues: issues.filter((i: SeoIssue) => i.severity === "warning").length,
    suggestion_issues: issues.filter((i: SeoIssue) => i.severity === "info").length,
    top_issues: issues.slice(0, 10).map((i: SeoIssue) => `- ${i.title}`),
    avg_load_time: summary.avg_load_time,
    total_words: summary.total_words,
    ssl_certificate: analysis.ssl_certificate,
    sitemap_found: analysis.sitemap_found,
    robots_txt_found: analysis.robots_txt_found,
  };
}

export type AiSource = "gemini" | "local";

/**
 * Generate AI insights using whichever source the user has selected in
 * Settings → AI. Returns { text, source } on success.
 */
export async function generateAnalysis(
  result: CompleteAnalysisResponse,
): Promise<Result<{ text: string; source: AiSource }, string>> {
  const payload = buildInsightsPayload(result);

  const sourceRes = await commands.getAiSource();
  const source: AiSource =
    sourceRes.status === "ok" && sourceRes.data === "local" ? "local" : "gemini";

  if (source === "gemini") {
    const res = await commands.getGeminiInsights(payload);
    if (res.status === "ok") return Result.Ok({ text: res.data, source: "gemini" });
    const err = res.status === "error" ? (res.error as string) : "Gemini request failed";
    handleAiUiEffects(mapErrorToType(err));
    return Result.Err(err);
  }

  // local
  const res = await commands.generateLocalInsights(payload);
  if (res.status === "ok") return Result.Ok({ text: res.data, source: "local" });
  return Result.Err(
    res.status === "error"
      ? (res.error as string)
      : "Local model inference failed. Make sure a model is downloaded and active in Settings → AI.",
  );
}

function handleAiUiEffects(error: AiError) {
  switch (error) {
    case AiError.MissingKey:
      toast("Gemini API Configuration", {
        description: "API key is missing.",
        action: { label: "Configure", onClick: () => openSettingsDialog() },
      });
      break;
    case AiError.InvalidKey:
      toast("Gemini API Configuration", {
        description: "The API key provided is invalid.",
        action: { label: "Configure", onClick: () => openSettingsDialog() },
      });
      break;

    case AiError.RateLimit:
      toast.error("Rate Limit Exceeded", {
        description: "The AI is currently busy (429). Please try again later.",
      });
      break;

    case AiError.NetworkError:
      toast.error("Connection Error", {
        description: "Could not reach Gemini services. Check your internet.",
      });
      break;

    default:
      toast.error("AI Analysis Failed", {
        description: "An unexpected error occurred.",
      });
  }
}
