import { PageAnalysisData } from "@/src/lib/types"
import { Separator } from "@radix-ui/react-dropdown-menu"
import { Card, CardContent } from "../../ui/card"
import HealthIndicator from "../atoms/HealthIndicator"
import { CompleteAnalysisResponse } from "@/src/lib/types"

export function SiteHealthCard({
    analysis,
    pages,
}: {
    analysis: CompleteAnalysisResponse["analysis"]
    pages: PageAnalysisData[]
}) {
    const mobilePages = pages.filter((p) => p.mobile_friendly).length
    const structuredDataPages = pages.filter((p) => p.has_structured_data).length

    return (
        <Card className="bg-card/40 backdrop-blur-sm border-white/5 shadow-sm relative group">
            <div className="absolute inset-0 bg-gradient-to-br from-blue-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
            <CardContent className="p-4 relative z-10">
                <h3 className="text-xs font-medium uppercase tracking-wider text-muted-foreground mb-4">Site Health</h3>

                <div className="grid grid-cols-3 gap-4 mb-6">
                    <HealthIndicator label="SSL" active={analysis.ssl_certificate} />
                    <HealthIndicator label="Sitemap" active={analysis.sitemap_found} />
                    <HealthIndicator label="robots.txt" active={analysis.robots_txt_found} />
                </div>

                <Separator className="bg-border/40 my-4" />

                <div className="space-y-4">
                    <div className="space-y-1.5">
                        <div className="flex justify-between text-xs">
                            <span className="text-muted-foreground">Mobile Friendly</span>
                            <span className="font-mono">{mobilePages}/{pages.length}</span>
                        </div>
                        <div className="h-1.5 w-full bg-muted/30 rounded-full overflow-hidden">
                            <div
                                className="h-full bg-blue-500 rounded-full transition-all duration-1000"
                                style={{ width: `${(mobilePages / pages.length) * 100}%` }}
                            />
                        </div>
                    </div>

                    <div className="space-y-1.5">
                        <div className="flex justify-between text-xs">
                            <span className="text-muted-foreground">Structured Data</span>
                            <span className="font-mono">{structuredDataPages}/{pages.length}</span>
                        </div>
                        <div className="h-1.5 w-full bg-muted/30 rounded-full overflow-hidden">
                            <div
                                className="h-full bg-purple-500 rounded-full transition-all duration-1000"
                                style={{ width: `${(structuredDataPages / pages.length) * 100}%` }}
                            />
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}

