import jsPDF from "jspdf"
import type { CompleteAnalysisResult } from "@/src/lib/types"
import { generateReport, generateCSV } from "@/src/lib/report-generator"
import { save } from "@tauri-apps/plugin-dialog"
import { writeTextFile, writeFile } from "@tauri-apps/plugin-fs"

import { toast } from "sonner"

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function getScoreLabel(score: number): string {
    if (score >= 90) return "Excellent"
    if (score >= 80) return "Good"
    if (score >= 60) return "Fair"
    if (score >= 40) return "Poor"
    return "Critical"
}

function getScoreColor(score: number): [number, number, number] {
    if (score >= 80) return [34, 197, 94] // green
    if (score >= 50) return [234, 179, 8] // yellow
    return [239, 68, 68] // red
}

function formatDomain(url: string): string {
    return url.replace(/^https?:\/\//, "").replace(/[^a-z0-9]/gi, "-")
}

function formatDate(date: Date = new Date()): string {
    return date.toISOString().split("T")[0]
}

/**
 * Save file using Tauri's save dialog (if available) or browser download
 */
async function saveFile(
    content: string | Uint8Array<ArrayBuffer>,
    defaultFilename: string,
    filters?: { name: string; extensions: string[] }[]
): Promise<void> {
    try {
        const filePath = await save({
            defaultPath: defaultFilename,
            filters: filters || [],
        })

        if (filePath) {
            if (typeof content === "string") {
                await writeTextFile(filePath, content)
            } else {
                // For binary data (like PDFs), use writeFile
                await writeFile(filePath, content)
            }
            toast.success("File saved successfully")
            console.log("File saved successfully:", filePath)
        } else {
            toast.info("File save was cancelled")
            console.log("Save cancelled by user")
        }
    } catch (error: any) {
        // Check for common cancellation errors
        const errorMessage = String(error)
        const errorString = JSON.stringify(error, Object.getOwnPropertyNames(error))

        if (
            errorMessage.includes("cancelled") ||
            errorMessage.includes("-999") ||
            errorMessage.includes("NSURLErrorDomain") ||
            errorMessage.includes("Operation couldn't be completed") ||
            errorString.includes("-999") ||
            (typeof error === 'object' && error !== null && 'code' in error && error.code === -999)
        ) {
            toast.info("Save cancelled")
            console.log("Save cancelled by user (caught error)")
            return
        }

        console.error("Error saving file:", error)
        toast.error("Failed to save file. Using browser download instead.")
        // Fallback to browser download if Tauri API fails
        fallbackDownload(content, defaultFilename)
    }
}

/**
 * Fallback to browser download if Tauri API is not available
 */
function fallbackDownload(content: string | Uint8Array<ArrayBuffer>, filename: string) {
    // Determine MIME type based on extension
    const extension = filename.split(".").pop()?.toLowerCase()
    let mimeType = "text/plain"

    if (extension === "pdf") {
        mimeType = "application/pdf"
    } else if (extension === "csv") {
        mimeType = "text/csv"
    } else if (extension === "txt") {
        mimeType = "text/plain"
    }

    const blob = new Blob([content], { type: mimeType })

    const url = URL.createObjectURL(blob)
    const a = document.createElement("a")
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    toast.success("File downloaded successfully")
    console.log("File downloaded successfully (fallback):", filename)
}

// ============================================================================
// PDF GENERATION
// ============================================================================

import { generateGeminiAnalysis } from "@/src/api/ai"
import { execute } from "@/src/lib/tauri"

export async function generatePDF(result: CompleteAnalysisResult): Promise<void> {
    const { analysis, summary, pages, issues } = result

    // Generate AI-powered recommendations if enabled
    const aiEnabledResult = await execute<boolean>("get_gemini_enabled")
    const aiEnabled = aiEnabledResult.unwrapOr(false)
    let aiInsights: string | null = null

    if (aiEnabled) {
        toast.info("Generating AI-powered insights...")
        aiInsights = await generateGeminiAnalysis(result)
    } else {
        toast.info("AI analysis skipped (disabled in settings)")
    }

    const pdf = new jsPDF()
    const pageWidth = pdf.internal.pageSize.getWidth()
    const pageHeight = pdf.internal.pageSize.getHeight()
    const margin = 20
    let y = margin

    // Helper to add new page if needed
    const checkPageBreak = (requiredSpace: number) => {
        if (y + requiredSpace > pageHeight - margin) {
            pdf.addPage()
            y = margin
            return true
        }
        return false
    }

    // Helper to add text with automatic line wrapping
    const addText = (text: string, x: number, fontSize: number = 10, options?: { maxWidth?: number; color?: [number, number, number] }) => {
        pdf.setFontSize(fontSize)
        if (options?.color) {
            pdf.setTextColor(...options.color)
        } else {
            pdf.setTextColor(0, 0, 0)
        }

        const lines = pdf.splitTextToSize(text, options?.maxWidth || pageWidth - 2 * margin)
        for (const line of lines) {
            checkPageBreak(fontSize / 2 + 2)
            pdf.text(line, x, y)
            y += fontSize / 2 + 2
        }
    }

    // ========================================================================
    // PROFESSIONAL HEADER WITH GRADIENT
    // ========================================================================
    // Gradient effect (simulate with multiple rectangles)
    const headerHeight = 45
    for (let i = 0; i < headerHeight; i++) {
        const ratio = i / headerHeight
        // Blue to indigo gradient
        const r = Math.floor(59 + (99 - 59) * ratio)
        const g = Math.floor(130 + (102 - 130) * ratio)
        const b = Math.floor(246 + (241 - 246) * ratio)
        pdf.setFillColor(r, g, b)
        pdf.rect(0, i, pageWidth, 1, "F")
    }

    // Title
    pdf.setTextColor(255, 255, 255)
    pdf.setFontSize(28)
    pdf.text("SEO Analysis Report", margin, 22)

    // Subtitle
    pdf.setFontSize(11)
    pdf.setTextColor(220, 220, 220)
    pdf.text("Comprehensive Website SEO Audit", margin, 32)

    y = 55

    // Report metadata card
    pdf.setFillColor(249, 250, 251) // gray-50
    pdf.setDrawColor(229, 231, 235) // gray-200
    pdf.setLineWidth(0.3)
    pdf.roundedRect(margin, y, pageWidth - 2 * margin, 18, 2, 2, "FD")

    pdf.setFontSize(10)
    pdf.setTextColor(75, 85, 99) // gray-600
    pdf.text(`Website: ${analysis.url}`, margin + 5, y + 7)
    pdf.text(`Generated: ${new Date().toLocaleDateString()} at ${new Date().toLocaleTimeString()}`, margin + 5, y + 14)

    y += 25

    // ========================================================================
    // SEO SCORE CARD (ENHANCED)
    // ========================================================================
    checkPageBreak(55)

    // Card with shadow effect
    pdf.setFillColor(255, 255, 255)
    pdf.setDrawColor(209, 213, 219) // gray-300
    pdf.setLineWidth(0.5)
    pdf.roundedRect(margin, y, pageWidth - 2 * margin, 48, 3, 3, "FD")

    const scoreColor = getScoreColor(summary.seo_score)

    // Large score number
    pdf.setFontSize(48)
    pdf.setTextColor(...scoreColor)
    pdf.text(`${summary.seo_score}`, margin + 15, y + 35)

    // Score label
    pdf.setFontSize(12)
    pdf.setTextColor(0, 0, 0)
    pdf.text("SEO Score", margin + 60, y + 20)

    pdf.setFontSize(14)
    pdf.setTextColor(...scoreColor)
    pdf.text(getScoreLabel(summary.seo_score), margin + 60, y + 32)

    // Visual progress bar
    const barWidth = 80
    const barX = margin + 60
    const barY = y + 38

    // Background bar
    pdf.setFillColor(229, 231, 235) // gray-200
    pdf.roundedRect(barX, barY, barWidth, 4, 2, 2, "F")

    // Progress bar
    const progressWidth = (summary.seo_score / 100) * barWidth
    pdf.setFillColor(...scoreColor)
    pdf.roundedRect(barX, barY, progressWidth, 4, 2, 2, "F")

    y += 58

    // ========================================================================
    // EXECUTIVE SUMMARY  (ENHANCED WITH METRIC CARDS)
    // ========================================================================
    checkPageBreak(65)

    pdf.setFontSize(18)
    pdf.setTextColor(31, 41, 55) // gray-800
    pdf.text("Executive Summary", margin, y)
    y += 12

    // Metric cards in grid
    const cardWidth = (pageWidth - 2 * margin - 10) / 2
    const cardHeight = 22

    const metrics = [
        { label: "Pages Analyzed", value: `${pages.length}`, color: [59, 130, 246] }, // blue
        { label: "Total Issues", value: `${summary.total_issues}`, color: [239, 68, 68] }, // red
        { label: "Avg Load Time", value: `${summary.avg_load_time.toFixed(2)}s`, color: [234, 179, 8] }, // yellow
        { label: "Total Words", value: summary.total_words.toLocaleString(), color: [34, 197, 94] }, // green
    ]

    metrics.forEach((metric, index) => {
        const col = index % 2
        const row = Math.floor(index / 2)
        const x = margin + col * (cardWidth + 10)
        const yPos = y + row * (cardHeight + 5)

        // Card background
        pdf.setFillColor(249, 250, 251) // gray-50
        pdf.setDrawColor(metric.color[0], metric.color[1], metric.color[2])
        pdf.setLineWidth(0.8)
        pdf.roundedRect(x, yPos, cardWidth, cardHeight, 2, 2, "FD")

        // Metric value
        pdf.setFontSize(20)
        pdf.setTextColor(metric.color[0], metric.color[1], metric.color[2])
        pdf.text(metric.value, x + 5, yPos + 12)

        // Metric label
        pdf.setFontSize(9)
        pdf.setTextColor(107, 114, 128) // gray-500
        pdf.text(metric.label, x + 5, yPos + 18)
    })

    y += (Math.ceil(metrics.length / 2) * (cardHeight + 5)) + 10

    // ========================================================================
    // SITE HEALTH
    // ========================================================================
    checkPageBreak(40)

    pdf.setFontSize(16)
    pdf.setTextColor(0, 0, 0)
    pdf.text("Site Health", margin, y)
    y += 10

    pdf.setFontSize(10)
    const healthItems = [
        { label: "SSL Certificate", value: analysis.ssl_certificate },
        { label: "Sitemap Found", value: analysis.sitemap_found },
        { label: "robots.txt Found", value: analysis.robots_txt_found },
    ]

    for (const item of healthItems) {
        checkPageBreak(6)
        const status = item.value ? "✓" : "✗"
        const color: [number, number, number] = item.value ? [34, 197, 94] : [239, 68, 68]
        pdf.setTextColor(...color)
        pdf.text(status, margin + 5, y)
        pdf.setTextColor(0, 0, 0)
        pdf.text(item.label, margin + 15, y)
        y += 6
    }

    y += 10

    // ========================================================================
    // AI-POWERED INSIGHTS
    // ========================================================================
    if (aiInsights) {
        checkPageBreak(60)

        // Section header with gradient background
        pdf.setFillColor(99, 102, 241) // indigo-500
        pdf.rect(margin, y, pageWidth - 2 * margin, 10, "F")

        pdf.setFontSize(16)
        pdf.setTextColor(255, 255, 255)
        pdf.text("AI-Powered Insights", margin + 5, y + 7)
        y += 15

        // AI content with accent border
        pdf.setDrawColor(99, 102, 241)
        pdf.setLineWidth(0.5)
        pdf.line(margin, y, margin, y + 50)

        pdf.setFontSize(9)
        pdf.setTextColor(50, 50, 50)
        addText(aiInsights, margin + 3, 9, { maxWidth: pageWidth - 2 * margin - 6 })

        y += 15
    }

    // ========================================================================
    // ISSUES BREAKDOWN
    // ========================================================================
    const criticalIssues = issues.filter((i) => i.issue_type === "Critical")
    const warningIssues = issues.filter((i) => i.issue_type === "Warning")
    const suggestionIssues = issues.filter((i) => i.issue_type === "Suggestion")

    // Critical Issues
    if (criticalIssues.length > 0) {
        checkPageBreak(20)

        pdf.setFontSize(14)
        pdf.setTextColor(239, 68, 68)
        pdf.text(`Critical Issues (${criticalIssues.length})`, margin, y)
        y += 8

        pdf.setFontSize(9)
        for (const issue of criticalIssues.slice(0, 5)) {
            checkPageBreak(15)
            pdf.setTextColor(0, 0, 0)
            addText(`• ${issue.title}`, margin + 5, 9, { maxWidth: pageWidth - 2 * margin - 10 })
            pdf.setTextColor(100, 100, 100)
            addText(`  Page: ${issue.page_url}`, margin + 7, 8, { maxWidth: pageWidth - 2 * margin - 12 })
            y += 2
        }

        if (criticalIssues.length > 5) {
            pdf.setTextColor(100, 100, 100)
            pdf.text(`  ...and ${criticalIssues.length - 5} more`, margin + 7, y)
            y += 6
        }

        y += 5
    }

    // Warning Issues
    if (warningIssues.length > 0) {
        checkPageBreak(20)

        pdf.setFontSize(14)
        pdf.setTextColor(234, 179, 8)
        pdf.text(`Warnings (${warningIssues.length})`, margin, y)
        y += 8

        pdf.setFontSize(9)
        for (const issue of warningIssues.slice(0, 3)) {
            checkPageBreak(15)
            pdf.setTextColor(0, 0, 0)
            addText(`• ${issue.title}`, margin + 5, 9, { maxWidth: pageWidth - 2 * margin - 10 })
            pdf.setTextColor(100, 100, 100)
            addText(`  Page: ${issue.page_url}`, margin + 7, 8, { maxWidth: pageWidth - 2 * margin - 12 })
            y += 2
        }

        if (warningIssues.length > 3) {
            pdf.setTextColor(100, 100, 100)
            pdf.text(`  ...and ${warningIssues.length - 3} more`, margin + 7, y)
            y += 6
        }

        y += 5
    }

    // ========================================================================
    // PAGE SUMMARY TABLE
    // ========================================================================
    checkPageBreak(30)

    pdf.setFontSize(14)
    pdf.setTextColor(0, 0, 0)
    pdf.text("Page Summary", margin, y)
    y += 10

    // Table header
    pdf.setFillColor(59, 130, 246)
    pdf.rect(margin, y, pageWidth - 2 * margin, 8, "F")
    pdf.setFontSize(8)
    pdf.setTextColor(255, 255, 255)
    pdf.text("URL", margin + 2, y + 5)
    pdf.text("Load Time", margin + 80, y + 5)
    pdf.text("Words", margin + 110, y + 5)
    pdf.text("Issues", margin + 135, y + 5)
    y += 10

    // Table rows
    pdf.setFontSize(7)
    pdf.setTextColor(0, 0, 0)
    for (const page of pages.slice(0, 15)) {
        checkPageBreak(8)

        const url = page.url.replace(/^https?:\/\/[^/]+/, "") || "/"
        const truncatedUrl = url.length > 35 ? url.substring(0, 32) + "..." : url
        const pageIssues = issues.filter((i) => i.page_url === page.url).length

        pdf.text(truncatedUrl, margin + 2, y)
        pdf.text(`${page.load_time.toFixed(2)}s`, margin + 80, y)
        pdf.text(`${page.word_count}`, margin + 110, y)
        pdf.text(`${pageIssues}`, margin + 135, y)

        // Alternate row background
        if (pages.indexOf(page) % 2 === 0) {
            pdf.setFillColor(250, 250, 250)
            pdf.rect(margin, y - 4, pageWidth - 2 * margin, 6, "F")
        }

        y += 6
    }

    if (pages.length > 15) {
        y += 4
        pdf.setTextColor(100, 100, 100)
        pdf.text(`...and ${pages.length - 15} more pages`, margin + 2, y)
    }

    // ========================================================================
    // FOOTER
    // ========================================================================
    const totalPages = pdf.internal.pages.length - 1
    for (let i = 1; i <= totalPages; i++) {
        pdf.setPage(i)
        pdf.setFontSize(8)
        pdf.setTextColor(150, 150, 150)
        pdf.text(`Page ${i} of ${totalPages}`, pageWidth - margin - 20, pageHeight - 10)
    }

    // Save the PDF
    const filename = `seo-report-${formatDomain(analysis.url)}-${formatDate()}.pdf`
    const pdfBlob = pdf.output("arraybuffer")
    const pdfData = new Uint8Array(pdfBlob)

    await saveFile(pdfData, filename, [
        { name: "PDF Files", extensions: ["pdf"] },
        { name: "All Files", extensions: ["*"] },
    ])
}

// ============================================================================
// TEXT EXPORT
// ============================================================================

export async function downloadTextReport(result: CompleteAnalysisResult): Promise<void> {
    const reportText = generateReport(result)
    const filename = `seo-report-${formatDomain(result.analysis.url)}-${formatDate()}.txt`

    await saveFile(reportText, filename, [
        { name: "Text Files", extensions: ["txt"] },
        { name: "All Files", extensions: ["*"] },
    ])
}

// ============================================================================
// CSV EXPORT
// ============================================================================

export async function downloadCSVReport(result: CompleteAnalysisResult): Promise<void> {
    const csvData = generateCSV(result)
    const filename = `seo-data-${formatDomain(result.analysis.url)}-${formatDate()}.csv`

    await saveFile(csvData, filename, [
        { name: "CSV Files", extensions: ["csv"] },
        { name: "All Files", extensions: ["*"] },
    ])
}
