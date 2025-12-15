import type { CompleteAnalysisResult } from "@/src/lib/types"

function getScoreLabel(score: number) {
    if (score >= 90) return "Excellent"
    if (score >= 80) return "Good"
    if (score >= 60) return "Fair"
    if (score >= 40) return "Poor"
    return "Critical"
}

export function generateReport(result: CompleteAnalysisResult): string {
    const { analysis, summary, pages, issues } = result
    const criticalIssues = issues.filter((i) => i.issue_type === "Critical")
    const warningIssues = issues.filter((i) => i.issue_type === "Warning")
    const suggestionIssues = issues.filter((i) => i.issue_type === "Suggestion")

    return `
SEO ANALYSIS REPORT
${"=".repeat(60)}

Website: ${analysis.url}
Generated: ${new Date().toLocaleString()}
Analysis Completed: ${analysis.completed_at ? new Date(analysis.completed_at).toLocaleString() : "N/A"}

${"=".repeat(60)}
EXECUTIVE SUMMARY
${"=".repeat(60)}

Overall SEO Score: ${summary.seo_score}/100 (${getScoreLabel(summary.seo_score)})
Pages Analyzed: ${pages.length}
Total Issues Found: ${summary.total_issues}
  - Critical: ${criticalIssues.length}
  - Warnings: ${warningIssues.length}
  - Suggestions: ${suggestionIssues.length}

Average Load Time: ${summary.avg_load_time.toFixed(2)}s
Total Word Count: ${summary.total_words.toLocaleString()}

Site Health:
  - SSL Certificate: ${analysis.ssl_certificate ? "Valid" : "Missing"}
  - Sitemap: ${analysis.sitemap_found ? "Found" : "Not Found"}
  - robots.txt: ${analysis.robots_txt_found ? "Found" : "Not Found"}

${"=".repeat(60)}
CRITICAL ISSUES (${criticalIssues.length})
${"=".repeat(60)}
${criticalIssues.length === 0
            ? "\nNo critical issues found.\n"
            : criticalIssues
                .map(
                    (issue, i) => `
${i + 1}. ${issue.title}
   Page: ${issue.page_url}
   Description: ${issue.description}
   Recommendation: ${issue.recommendation}
`,
                )
                .join("")
        }

${"=".repeat(60)}
WARNINGS (${warningIssues.length})
${"=".repeat(60)}
${warningIssues.length === 0
            ? "\nNo warnings found.\n"
            : warningIssues
                .map(
                    (issue, i) => `
${i + 1}. ${issue.title}
   Page: ${issue.page_url}
   Description: ${issue.description}
   Recommendation: ${issue.recommendation}
`,
                )
                .join("")
        }

${"=".repeat(60)}
SUGGESTIONS (${suggestionIssues.length})
${"=".repeat(60)}
${suggestionIssues.length === 0
            ? "\nNo suggestions.\n"
            : suggestionIssues
                .map(
                    (issue, i) => `
${i + 1}. ${issue.title}
   Page: ${issue.page_url}
   Description: ${issue.description}
   Recommendation: ${issue.recommendation}
`,
                )
                .join("")
        }

${"=".repeat(60)}
PAGE-BY-PAGE ANALYSIS
${"=".repeat(60)}
${pages
            .map(
                (page, i) => `
${i + 1}. ${page.url}
   Title: ${page.title || "Missing"}
   Meta Description: ${page.meta_description ? "Present" : "Missing"}
   Load Time: ${page.load_time.toFixed(2)}s
   Word Count: ${page.word_count}
   Headings: H1(${page.h1_count}) H2(${page.h2_count}) H3(${page.h3_count})
   Images: ${page.image_count} (${page.images_without_alt} missing alt)
   Links: ${page.internal_links} internal, ${page.external_links} external
   Mobile Friendly: ${page.mobile_friendly ? "Yes" : "No"}
   Structured Data: ${page.has_structured_data ? "Yes" : "No"}
   ${page.lighthouse_seo ? `Lighthouse SEO: ${page.lighthouse_seo}/100` : ""}
`,
            )
            .join("")}

${"=".repeat(60)}
END OF REPORT
${"=".repeat(60)}
`.trim()
}

export function generateCSV(result: CompleteAnalysisResult): string {
    const { pages, issues } = result
    const header = [
        "URL",
        "Status",
        "Load Time (s)",
        "Word Count",
        "H1",
        "H2",
        "H3",
        "Images",
        "Missing Alt",
        "Int Links",
        "Ext Links",
        "Mobile Friendly",
        "Issues Found",
    ].join(",")

    const rows = pages.map((p) => {
        // Count issues for this page - usually matched by page_url
        const issueCount = issues.filter((i) => i.page_url === p.url).length

        // Escape URL to prevent CSV injection or formatting errors
        const safeUrl = `"${p.url.replace(/"/g, '""')}"`

        return [
            safeUrl,
            p.status_code || "N/A",
            p.load_time.toFixed(2),
            p.word_count,
            p.h1_count,
            p.h2_count,
            p.h3_count,
            p.image_count,
            p.images_without_alt,
            p.internal_links,
            p.external_links,
            p.mobile_friendly ? "Yes" : "No",
            issueCount,
        ].join(",")
    })

    return [header, ...rows].join("\n")
}
