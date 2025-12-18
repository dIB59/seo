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
    const criticalCount = issues.filter((i) => i.issue_type === "Critical").length
    const warningCount = issues.filter((i) => i.issue_type === "Warning").length
    const suggestionCount = issues.filter((i) => i.issue_type === "Suggestion").length

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
                                <IssueBadge type="Critical" count={criticalCount} />
                                <IssueBadge type="Warning" count={warningCount} />
                                <IssueBadge type="Suggestion" count={suggestionCount} />
                            </div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}
