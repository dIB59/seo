import { cn } from "@/src/lib/utils"
import { Card, CardContent } from "../../ui/card"
import { Separator } from "../../ui/separator"
import { ScoreRing } from "../atoms/ScoreRing"
import { IssueBadge } from "../atoms/IssueBadge"
import { getScoreColor, getScoreLabel } from "@/src/lib/seo-metrics"
import { CompleteAnalysisResponse, SeoIssueResponse } from "@/src/lib/types"

export function ScoreCard({
    summary,
    issues,
}: {
    summary: CompleteAnalysisResponse["summary"]
    issues: SeoIssueResponse[]
}) {
    const criticalCount = issues.filter((i) => i.severity === "critical").length
    const warningCount = issues.filter((i) => i.severity === "warning").length
    const suggestionCount = issues.filter((i) => i.severity === "info").length
    return (
        <Card className="bg-card/40 backdrop-blur-sm border-white/5 shadow-sm overflow-hidden relative group">
            <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
            <CardContent className="p-6 relative z-10">
                <div className="flex items-center gap-6">
                    <div className="relative">
                        {/* Glow behind ring */}
                        <div className={cn("absolute inset-0 blur-2xl opacity-20 rounded-full", getScoreColor(summary.seo_score).replace("text-", "bg-"))} />
                        <ScoreRing score={summary.seo_score} size="lg" />
                    </div>
                    <div className="flex-1 min-w-0 space-y-3">
                        <div>
                            <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground mb-1">Overall Score</p>
                            <div className="flex items-baseline gap-2">
                                <h3 className={cn("text-4xl font-bold tracking-tighter font-mono", getScoreColor(summary.seo_score))}>
                                    {summary.seo_score}
                                </h3>
                                <span className="text-sm font-medium opacity-80">{getScoreLabel(summary.seo_score)}</span>
                            </div>
                        </div>
                        <Separator className="bg-border/40" />
                        <div className="pt-1">
                            <div className="flex items-center gap-2 mb-2">
                                <p className="text-xs text-muted-foreground">Issues Found</p>
                                <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-muted/40 text-muted-foreground font-mono">{issues.length}</span>
                            </div>
                            <div className="flex flex-wrap gap-2">
                                <IssueBadge type="critical" count={criticalCount} />
                                <IssueBadge type="warning" count={warningCount} />
                                <IssueBadge type="info" count={suggestionCount} />
                            </div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}
