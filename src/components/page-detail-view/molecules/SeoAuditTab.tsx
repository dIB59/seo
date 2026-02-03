import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card"
import type { PageDetailData } from "@/src/lib/types"
import ScoreOverview from "./ScoreOverview"
import PerformanceMetrics from "./PerformanceMetrics"
import SeoAuditBreakdown from "./SeoAuditBreakdown"
import { Search, Activity } from "lucide-react"

export default function SeoAuditTab({ page }: { page: PageDetailData }) {
    const seoAudits = page.lighthouse_seo_audits
    const perfMetrics = page.lighthouse_performance_metrics

    if (!page.lighthouse_seo) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <Search className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No SEO audit data available</p>
                    <p className="text-sm text-muted-foreground mt-1">Run an analysis to see SEO scores</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <div className="space-y-4">
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="flex items-center gap-2 text-base"><Search className="h-4 w-4" />{perfMetrics ? "Deep Audit Scores" : "SEO Scores"}</CardTitle>
                </CardHeader>
                <CardContent>
                    <ScoreOverview page={page} />
                </CardContent>
            </Card>

            {perfMetrics && (
                <Card>
                    <CardHeader className="pb-2">
                        <CardTitle className="flex items-center gap-2 text-base"><Activity className="h-4 w-4" />Core Web Vitals & Performance Metrics</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <PerformanceMetrics metrics={perfMetrics} />
                    </CardContent>
                </Card>
            )}

            {seoAudits && (
                <Card>
                    <CardHeader className="pb-2">
                        <CardTitle className="flex items-center gap-2 text-base"><Search className="h-4 w-4" />SEO Audit Breakdown</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <SeoAuditBreakdown audits={seoAudits} />
                    </CardContent>
                </Card>
            )}
        </div>
    )
}
