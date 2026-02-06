import { CompleteAnalysisResult, SeoIssue } from "@/src/lib/types"
import { cn } from "@/src/lib/utils"
import { Card, CardContent } from "../../ui/card"
import { Separator } from "../../ui/separator"
import { ScoreRing } from "../atoms/ScoreRing"
import { IssueBadge } from "../atoms/IssueBadge"
import { getScoreColor, getScoreLabel } from "@/src/lib/seo-metrics"

export function ScoreCard({
    summary,
    issues,
}: {
    summary: CompleteAnalysisResult["summary"]
    issues: SeoIssue[]
}) {
    const criticalCount = issues.filter((i) => i.severity === "critical").length
    const warningCount = issues.filter((i) => i.severity === "warning").length
    const suggestionCount = issues.filter((i) => i.severity === "info").length
    return (
        <Card>
            <CardContent className="p-6">
                <div className="flex items-start gap-6">
                    <ScoreRing score={summary.seo_score} size="lg" />
                    <div className="flex-1 min-w-0 space-y-2">
                        <div>
                            <h3 className="text-lg font-semibold">SEO Score</h3>
                            <p className={cn("text-sm font-medium", getScoreColor(summary.seo_score))}>
                                {getScoreLabel(summary.seo_score)}
                            </p>
                        </div>
                        <Separator />
                        <div className="pt-1">
                            <p className="text-xs text-muted-foreground mb-2">Issues Found</p>
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
