import { PageAnalysisData, SeoIssue } from "@/src/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card";
import { ProgressRow } from "../atoms/ProgressRow";

export function OverviewTab({ issues, pages }: { issues: SeoIssue[]; pages: PageAnalysisData[] }) {
    const criticalCount = issues.filter((i) => i.severity === "critical").length
    const warningCount = issues.filter((i) => i.severity === "warning").length
    const suggestionCount = issues.filter((i) => i.severity === "info").length

    const fastPages = pages.filter((p) => p.load_time < 1).length
    const mediumPages = pages.filter((p) => p.load_time >= 1 && p.load_time < 2).length
    const slowPages = pages.filter((p) => p.load_time >= 2).length

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Card>
                <CardHeader>
                    <CardTitle className="text-sm">Issue Distribution</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="space-y-3">
                        <ProgressRow label="Critical" value={criticalCount} total={issues.length} color="destructive" />
                        <ProgressRow label="Warning" value={warningCount} total={issues.length} color="warning" />
                        <ProgressRow label="Suggestion" value={suggestionCount} total={issues.length} color="primary" />
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle className="text-sm">Performance Summary</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="space-y-3">
                        <ProgressRow label="Fast (<1s)" value={fastPages} total={pages.length} color="success" />
                        <ProgressRow label="Medium (1-2s)" value={mediumPages} total={pages.length} color="warning" />
                        <ProgressRow label="Slow (>2s)" value={slowPages} total={pages.length} color="destructive" />
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}