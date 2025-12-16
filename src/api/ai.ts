import { execute } from "@/src/lib/tauri"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import { toast } from "sonner"

/**
 * Get Gemini API key from database 
 */
async function getStoredApiKey(): Promise<string | null> {
    try {
        const existingKeyResult = await execute<string | null>("get_gemini_api_key")
        const existingKey = existingKeyResult.expect("Failed to retrieve API key")
        return existingKey && existingKey.trim().length > 0 ? existingKey : null
    } catch (error) {
        console.error("Error checking API key:", error)
        return null
    }
}

/**
 * Trigger settings dialog (opens on API Key tab by default due to missing key)
 */
function openSettingsDialog() {
    window.dispatchEvent(new CustomEvent("open-settings-dialog"))
}

/**
 * Generate AI-powered SEO analysis using Google Gemini (via secure Tauri backend)
 */
export async function generateGeminiAnalysis(
    result: CompleteAnalysisResult
): Promise<string | null> {
    try {
        // Check for API key
        const apiKey = await getStoredApiKey()

        if (!apiKey) {
            toast("Gemini API Key Missing", {
                description: "AI insights will be skipped. Please configure your API key.",
                action: {
                    label: "Configure Key",
                    onClick: () => openSettingsDialog(),
                },
                duration: 10000,
            })
            // Automatically open dialog for better UX
            openSettingsDialog()
            return null
        }

        const { analysis, summary, issues, pages } = result

        // Prepare analysis summary
        const criticalIssues = issues.filter(i => i.issue_type === "Critical").length
        const warningIssues = issues.filter(i => i.issue_type === "Warning").length
        const suggestionIssues = issues.filter(i => i.issue_type === "Suggestion").length

        const topIssues = issues
            .slice(0, 10)
            .map(i => `- ${i.title} (${i.issue_type})`)

        // Call secure Tauri backend command
        const insightsResult = await execute<string>("get_gemini_insights", {
            analysisId: analysis.id,
            url: analysis.url,
            seoScore: summary.seo_score,
            pagesCount: pages.length,
            totalIssues: summary.total_issues,
            criticalIssues,
            warningIssues,
            suggestionIssues,
            topIssues,
            avgLoadTime: summary.avg_load_time,
            totalWords: summary.total_words,
            sslCertificate: analysis.ssl_certificate,
            sitemapFound: analysis.sitemap_found,
            robotsTxtFound: analysis.robots_txt_found,
        })
        const insights = insightsResult.expect("Failed to generate AI insights")

        return insights
    } catch (error) {
        console.error("Error generating Gemini analysis:", error)

        // Check if it's a missing API key error (backend might still throw if db check passed but key was invalid or emptied)
        const errorMessage = String(error)
        if (errorMessage.includes("API_KEY_MISSING")) {
            toast("Gemini API Key Missing", {
                description: "AI insights will be skipped. Configure API key to enable them.",
                action: {
                    label: "Configure Key",
                    onClick: () => openSettingsDialog(),
                },
                duration: 10000,
            })
            openSettingsDialog()
        } else {
            toast.error("Failed to generate AI insights", {
                description: "Please try again later",
            })
        }

        return null
    }
}
