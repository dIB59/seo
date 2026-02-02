import { PageAnalysisData } from "@/src/lib/types"
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card"
import { ScoreRing } from "../atoms/ScoreRing"
import { Gauge, Zap, Eye, Shield, Search } from "lucide-react"

interface LighthouseSummaryCardProps {
    pages: PageAnalysisData[]
}

function calculateAverage(pages: PageAnalysisData[], key: keyof PageAnalysisData): number | null {
    const values = pages
        .map((p) => p[key])
        .filter((v): v is number => v !== null && v !== undefined && typeof v === "number")

    if (values.length === 0) return null
    return Math.round(values.reduce((sum, v) => sum + v, 0) / values.length)
}

export function LighthouseSummaryCard({ pages }: LighthouseSummaryCardProps) {
    console.log("LighthouseSummaryCard pages:", pages[0].lighthouse_seo);   
    const avgPerformance = calculateAverage(pages, "lighthouse_performance")
    const avgAccessibility = calculateAverage(pages, "lighthouse_accessibility")
    const avgBestPractices = calculateAverage(pages, "lighthouse_best_practices")
    const avgSeo = calculateAverage(pages, "lighthouse_seo")

    // If no Lighthouse data, don't show the card
    if (avgPerformance === null && avgAccessibility === null && avgBestPractices === null && avgSeo === null) {
        return null
    }

    const pagesWithLighthouse = pages.filter((p) => p.lighthouse_performance !== null).length

    const scores = [
        { label: "Performance", value: avgPerformance, icon: Zap, color: "text-orange-500" },
        { label: "Accessibility", value: avgAccessibility, icon: Eye, color: "text-blue-500" },
        { label: "Best Practices", value: avgBestPractices, icon: Shield, color: "text-purple-500" },
        { label: "SEO", value: avgSeo, icon: Search, color: "text-green-500" },
    ]

    return (
        <Card>
            <CardHeader className="pb-2">
                <CardTitle className="flex items-center gap-2 text-base">
                    <Gauge className="h-4 w-4" />
                    Lighthouse Scores
                </CardTitle>
                <p className="text-xs text-muted-foreground">
                    Average across {pagesWithLighthouse} {pagesWithLighthouse === 1 ? "page" : "pages"}
                </p>
            </CardHeader>
            <CardContent className="pt-0">
                <div className="grid grid-cols-2 gap-3">
                    {scores.map((score) => {
                        const Icon = score.icon
                        return (
                            <div
                                key={score.label}
                                className="flex items-center gap-2 p-2 rounded-lg bg-muted/50"
                            >
                                <ScoreRing score={score.value ?? 0} size="sm" />
                                <div className="flex flex-col min-w-0">
                                    <div className="flex items-center gap-1">
                                        <Icon className={`h-3 w-3 ${score.color}`} />
                                        <span className="text-xs font-medium truncate">{score.label}</span>
                                    </div>
                                    {score.value === null && (
                                        <span className="text-[10px] text-muted-foreground">No data</span>
                                    )}
                                </div>
                            </div>
                        )
                    })}
                </div>
            </CardContent>
        </Card>
    )
}
