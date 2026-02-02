import { execute } from "@/src/lib/tauri"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import { toast } from "sonner"
import { Result } from "../lib/result";

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
    const existingKeyResult = await execute<string | null>("get_gemini_api_key")

    return existingKeyResult.andThen(key => {
        if (key && key.trim().length > 0) {
            return Result.Ok(key);
        }
        return Result.Err("API_KEY_MISSING");
    });
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
    result: CompleteAnalysisResult
): Promise<Result<string, AiError>> {

    const apiKeyResult = await getStoredApiKey();

    if (apiKeyResult.isErr() || !apiKeyResult.unwrap()) {
        handleAiUiEffects(AiError.MissingKey);
        return Result.Err(AiError.MissingKey);
    }

    const { analysis, summary, issues, pages } = result;

    // 2. Call Backend
    const insightsResult = await execute<string>("get_gemini_insights", {
        request: {
            analysisId: analysis.id,
            url: analysis.url,
            seoScore: summary.seo_score,
            pagesCount: pages.length,
            totalIssues: summary.total_issues,
            criticalIssues: issues.filter(i => i.issue_type === "critical").length,
            warningIssues: issues.filter(i => i.issue_type === "warning").length,
            suggestionIssues: issues.filter(i => i.issue_type === "suggestion").length,
            topIssues: issues.slice(0, 10).map(i => `- ${i.title}`),
            avgLoadTime: summary.avg_load_time,
            totalWords: summary.total_words,
            sslCertificate: analysis.ssl_certificate,
            sitemapFound: analysis.sitemap_found,
            robotsTxtFound: analysis.robots_txt_found,
        }
    });

    return insightsResult.match<Result<string, AiError>>(
        (data) => Result.Ok(data),
        (err) => {
            const errorType = mapErrorToType(err);
            handleAiUiEffects(errorType);
            return Result.Err(errorType);
        }
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
