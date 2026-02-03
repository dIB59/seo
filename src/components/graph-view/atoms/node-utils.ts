import type { CompleteAnalysisResult } from "@/src/lib/types"

export const calculateNodeDegrees = (
    pages: CompleteAnalysisResult['pages'],
    validUrls: Map<string, string>,
) => {
    const inDegree = new Map<string, number>()
    const outDegree = new Map<string, number>()

    pages.forEach(page => {
        inDegree.set(page.url, 0)
        outDegree.set(page.url, 0)
    })

    pages.forEach(page => {
        if (!page.detailed_links) return

        let outgoingCount = 0

        page.detailed_links.forEach(link => {
            if (!link.is_internal) return

            const targetUrl = (
                // caller can import resolveInternalUrl directly when composing
                // but for convenience we try to use validUrls map here
                validUrls.get(link.href) || null
            )
            if (targetUrl) {
                outgoingCount++
                inDegree.set(targetUrl, (inDegree.get(targetUrl) || 0) + 1)
            }
        })

        outDegree.set(page.url, outgoingCount)
    })

    return { inDegree, outDegree }
}

export const getNodeColor = (issues: CompleteAnalysisResult['issues'], pageUrl: string) => {
    const pageIssues = issues.filter(issue => issue.page_url === pageUrl)

    if (pageIssues.some(i => i.severity === "critical")) return '#f14444ff'
    if (pageIssues.some(i => i.severity === "warning")) return '#e8aa3fff'

    return '#46c773ff'
}

export const calculateNodeSize = (inDegree: number) => {
    return 2 + Math.log(inDegree + 1) * 2
}
