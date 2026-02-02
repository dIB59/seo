import { PageAnalysisData } from "@/src/lib/types"
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card"
import { ScoreRing } from "../atoms/ScoreRing"
import { Search, Eye, Shield, Zap } from "lucide-react"

interface SeoSummaryCardProps {
    pages: PageAnalysisData[]
}

function calculateAverage(pages: PageAnalysisData[], key: keyof PageAnalysisData): number | null {
    const values = pages
        .map((p) => p[key])
        .filter((v): v is number => v !== null && v !== undefined && typeof v === "number")

    if (values.length === 0) return null
    return Math.round(values.reduce((sum, v) => sum + v, 0) / values.length)
}

export function SeoSummaryCard({ pages }: SeoSummaryCardProps) {
    const avgSeo = calculateAverage(pages, "lighthouse_seo")
    const avgAccessibility = calculateAverage(pages, "lighthouse_accessibility")
    const avgBestPractices = calculateAverage(pages, "lighthouse_best_practices")
    const avgPerformance = calculateAverage(pages, "lighthouse_performance")

    // Count pages with SEO scores (always available from Light or Deep audit)
    const pagesWithScores = pages.filter((p) => p.lighthouse_seo !== null).length
    
    // Determine if we have deep audit data (performance metrics only from deep audit)
    const hasDeepAuditData = avgPerformance !== null

    const scores = [
        { label: "SEO", value: avgSeo, icon: Search, color: "text-green-500", alwaysShow: true },
        { label: "Accessibility", value: avgAccessibility, icon: Eye, color: "text-blue-500", alwaysShow: true },
        { label: "Best Practices", value: avgBestPractices, icon: Shield, color: "text-purple-500", alwaysShow: true },
        { label: "Performance", value: avgPerformance, icon: Zap, color: "text-orange-500", alwaysShow: false },
    ]

    // Filter scores based on what data we have
    const displayScores = hasDeepAuditData 
        ? scores 
        : scores.filter(s => s.alwaysShow)

    return (
        <Card>
            <CardHeader className="pb-2">
                <CardTitle className="flex items-center gap-2 text-base">
                    <Search className="h-4 w-4" />
                    {hasDeepAuditData ? "Deep Audit Scores" : "SEO Scores"}
                </CardTitle>
                <p className="text-xs text-muted-foreground">
                    Average across {pagesWithScores} {pagesWithScores === 1 ? "page" : "pages"}
                    {!hasDeepAuditData && " • Enable Deep Audit for performance metrics"}
                </p>
            </CardHeader>
            <CardContent className="pt-0">
                <div className={`grid gap-3 ${hasDeepAuditData ? 'grid-cols-2' : 'grid-cols-3'}`}>
                    {displayScores.map((score) => {
                        const Icon = score.icon
                        const hasValue = score.value !== null
                        return (
                            <div
                                key={score.label}
                                className="flex items-center gap-2 p-2 rounded-lg bg-muted/50"
                            >
                                {hasValue ? (
                                    <ScoreRing score={score.value} size="sm" />
                                ) : (
                                    <div className="w-8 h-8 rounded-full bg-muted flex items-center justify-center">
                                        <span className="text-[10px] text-muted-foreground">N/A</span>
                                    </div>
                                )}
                                <div className="flex flex-col min-w-0">
                                    <div className="flex items-center gap-1">
                                        <Icon className={`h-3 w-3 ${score.color}`} />
                                        <span className="text-xs font-medium truncate">{score.label}</span>
                                    </div>
                                </div>
                            </div>
                        )
                    })}
                </div>
            </CardContent>
        </Card>
    )
}
