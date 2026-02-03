import { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types"
import { Separator } from "@radix-ui/react-dropdown-menu"
import { Card, CardContent } from "../../ui/card"
import HealthIndicator from "../atoms/HealthIndicator"

export function SiteHealthCard({
    analysis,
    pages,
}: {
    analysis: CompleteAnalysisResult["analysis"]
    pages: PageAnalysisData[]
}) {
    const mobilePages = pages.filter((p) => p.mobile_friendly).length
    const structuredDataPages = pages.filter((p) => p.has_structured_data).length

    return (
        <Card>
            <CardContent className="p-6">
                <h3 className="text-sm font-semibold mb-3">Site Health</h3>
                <div className="grid grid-cols-3 gap-3">
                    <HealthIndicator label="SSL" active={analysis.ssl_certificate} />
                    <HealthIndicator label="Sitemap" active={analysis.sitemap_found} />
                    <HealthIndicator label="robots.txt" active={analysis.robots_txt_found} />
                </div>
                <Separator className="my-3" />
                <div className="grid grid-cols-2 gap-2 text-sm">
                    <div className="flex justify-between">
                        <span className="text-muted-foreground">Mobile Ready</span>
                        <span className={mobilePages === pages.length ? "text-success" : "text-warning"}>
                            {mobilePages}/{pages.length}
                        </span>
                    </div>
                    <div className="flex justify-between">
                        <span className="text-muted-foreground">Structured Data</span>
                        <span className={structuredDataPages === pages.length ? "text-success" : "text-muted-foreground"}>
                            {structuredDataPages}/{pages.length}
                        </span>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}

