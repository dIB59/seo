import { toast } from "sonner"
import { Result } from "../lib/result";
import type { CompleteAnalysisResponse, SeoIssue } from "@/src/lib/types";
import { commands } from "@/src/bindings";


export const AiError = {
    MissingKey: "MISSING_KEY",
    InvalidKey: "INVALID_KEY",
    RateLimit: "RATE_LIMIT",
    NetworkError: "NETWORK_ERROR",
    Unknown: "UNKNOWN",
} as const;

// This creates a type union: "MISSING_KEY" | "INVALID_KEY" | ...
export type AiError = typeof AiError[keyof typeof AiError];

/**
 * Get Gemini API key from database 
 */
async function getStoredApiKey(): Promise<Result<string, string>> {
    const res = await commands.getGeminiApiKey()
    if (res.status === "ok") {
        const key = res.data
        if (key && key.trim().length > 0) return Result.Ok(key)
        return Result.Err("API_KEY_MISSING")
    }

    return Result.Err(res.error ?? "API_KEY_MISSING")
}

/**
 * Trigger settings dialog (opens on API Key tab by default due to missing key)
 */
function openSettingsDialog() {
    window.dispatchEvent(new CustomEvent("open-settings-dialog"))
}

/**
 * Map backend error strings to our strict type
 */
function mapErrorToType(error: string): AiError {
    if (error.includes("API_KEY_MISSING")) return AiError.MissingKey;
    if (error.includes("401")) return AiError.InvalidKey;
    if (error.includes("429")) return AiError.RateLimit;
    return AiError.Unknown;
}

export async function generateGeminiAnalysis(
    result: CompleteAnalysisResponse
): Promise<Result<string, AiError>> {

    const apiKeyResult = await getStoredApiKey();

    if (apiKeyResult.isErr() || !apiKeyResult.unwrap()) {
        handleAiUiEffects(AiError.MissingKey);
        return Result.Err(AiError.MissingKey);
    }

    const { analysis, summary, issues, pages } = result;

    // 2. Call Backend via generated bindings
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
