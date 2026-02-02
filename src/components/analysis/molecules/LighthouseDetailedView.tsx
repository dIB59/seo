import { PageAnalysisData, LighthouseSeoAudits, LighthousePerformanceMetrics, LighthouseAuditResult } from "@/src/lib/types"
import { Zap, Eye, Shield, Search, CheckCircle2, XCircle, Clock, Gauge, Activity, LayoutPanelTop, MousePointer, FileText, Link2, Image, Globe, Bot, ChevronDown } from "lucide-react"
import { ScoreRing } from "../atoms/ScoreRing"
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card"
import { Badge } from "../../ui/badge"
import { cn } from "@/src/lib/utils"
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "../../ui/collapsible"
import { useState } from "react"

interface LighthouseDetailedViewProps {
    page: PageAnalysisData
}

// Format milliseconds to readable format
function formatTime(ms: number | null): string {
    if (ms === null) return "N/A"
    if (ms < 1000) return `${Math.round(ms)}ms`
    return `${(ms / 1000).toFixed(1)}s`
}

// Get color based on performance metric thresholds
function getMetricColor(metric: string, value: number | null): string {
    if (value === null) return "text-muted-foreground"
    
    // Thresholds based on Lighthouse recommendations
    const thresholds: Record<string, { good: number; moderate: number }> = {
        first_contentful_paint: { good: 1800, moderate: 3000 },
        largest_contentful_paint: { good: 2500, moderate: 4000 },
        speed_index: { good: 3400, moderate: 5800 },
        time_to_interactive: { good: 3800, moderate: 7300 },
        total_blocking_time: { good: 200, moderate: 600 },
        cumulative_layout_shift: { good: 0.1, moderate: 0.25 },
    }
    
    const threshold = thresholds[metric]
    if (!threshold) return "text-muted-foreground"
    
    if (value <= threshold.good) return "text-success"
    if (value <= threshold.moderate) return "text-warning"
    return "text-destructive"
}

// Audit status badge component
function AuditBadge({ audit, label }: { audit: LighthouseAuditResult; label: string }) {
    return (
        <div className="flex items-center justify-between py-2 px-3 rounded-lg bg-muted/30 hover:bg-muted/50 transition-colors">
            <div className="flex items-center gap-2">
                {audit.passed ? (
                    <CheckCircle2 className="h-4 w-4 text-success" />
                ) : (
                    <XCircle className="h-4 w-4 text-destructive" />
                )}
                <span className="text-sm">{label}</span>
            </div>
            <Badge 
                variant="outline" 
                className={cn(
                    "text-xs",
                    audit.passed 
                        ? "bg-success/15 text-success border-success/20" 
                        : "bg-destructive/15 text-destructive border-destructive/20"
                )}
            >
                {audit.passed ? "Passed" : "Failed"}
            </Badge>
        </div>
    )
}

// Performance metric row component
function MetricRow({ label, value, metric, icon: Icon }: { 
    label: string
    value: number | null
    metric: string
    icon: React.ComponentType<{ className?: string }>
}) {
    const isCLS = metric === "cumulative_layout_shift"
    const displayValue = isCLS ? (value?.toFixed(3) ?? "N/A") : formatTime(value)
    const colorClass = getMetricColor(metric, value)
    
    return (
        <div className="flex items-center justify-between py-2 px-3 rounded-lg bg-muted/30">
            <div className="flex items-center gap-2">
                <Icon className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm">{label}</span>
            </div>
            <span className={cn("text-sm font-medium", colorClass)}>
                {displayValue}
            </span>
        </div>
    )
}

export function LighthouseDetailedView({ page }: LighthouseDetailedViewProps) {
    const [seoOpen, setSeoOpen] = useState(true)
    const [perfOpen, setPerfOpen] = useState(true)
    
    // Only show if we have Lighthouse data
    if (!page.lighthouse_performance && !page.lighthouse_seo) return null

    const scores = [
        { label: "Performance", value: page.lighthouse_performance, icon: Zap, color: "text-orange-500" },
        { label: "Accessibility", value: page.lighthouse_accessibility, icon: Eye, color: "text-blue-500" },
        { label: "Best Practices", value: page.lighthouse_best_practices, icon: Shield, color: "text-purple-500" },
        { label: "SEO", value: page.lighthouse_seo, icon: Search, color: "text-green-500" },
    ]

    const seoAudits = page.lighthouse_seo_audits
    const perfMetrics = page.lighthouse_performance_metrics

    return (
        <div className="space-y-4">
            {/* Main Scores */}
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="flex items-center gap-2 text-base">
                        <Gauge className="h-4 w-4" />
                        Lighthouse Scores
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="grid grid-cols-4 gap-4">
                        {scores.map((score) => {
                            const Icon = score.icon
                            return (
                                <div
                                    key={score.label}
                                    className="flex flex-col items-center gap-2 p-3 rounded-lg bg-muted/50"
                                >
                                    <ScoreRing score={score.value || 0} size="md" />
                                    <div className="flex items-center gap-1">
                                        <Icon className={cn("h-3 w-3", score.color)} />
                                        <span className="text-xs font-medium">{score.label}</span>
                                    </div>
                                </div>
                            )
                        })}
                    </div>
                </CardContent>
            </Card>

            {/* Performance Metrics */}
            {perfMetrics && (
                <Collapsible open={perfOpen} onOpenChange={setPerfOpen}>
                    <Card>
                        <CollapsibleTrigger asChild>
                            <CardHeader className="pb-2 cursor-pointer hover:bg-muted/30 transition-colors">
                                <CardTitle className="flex items-center justify-between text-base">
                                    <div className="flex items-center gap-2">
                                        <Activity className="h-4 w-4" />
                                        Core Web Vitals & Performance Metrics
                                    </div>
                                    <ChevronDown className={cn(
                                        "h-4 w-4 transition-transform",
                                        perfOpen && "rotate-180"
                                    )} />
                                </CardTitle>
                            </CardHeader>
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                            <CardContent className="pt-0">
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                                    <MetricRow 
                                        label="First Contentful Paint (FCP)" 
                                        value={perfMetrics.first_contentful_paint}
                                        metric="first_contentful_paint"
                                        icon={Clock}
                                    />
                                    <MetricRow 
                                        label="Largest Contentful Paint (LCP)" 
                                        value={perfMetrics.largest_contentful_paint}
                                        metric="largest_contentful_paint"
                                        icon={LayoutPanelTop}
                                    />
                                    <MetricRow 
                                        label="Speed Index" 
                                        value={perfMetrics.speed_index}
                                        metric="speed_index"
                                        icon={Gauge}
                                    />
                                    <MetricRow 
                                        label="Time to Interactive (TTI)" 
                                        value={perfMetrics.time_to_interactive}
                                        metric="time_to_interactive"
                                        icon={MousePointer}
                                    />
                                    <MetricRow 
                                        label="Total Blocking Time (TBT)" 
                                        value={perfMetrics.total_blocking_time}
                                        metric="total_blocking_time"
                                        icon={Clock}
                                    />
                                    <MetricRow 
                                        label="Cumulative Layout Shift (CLS)" 
                                        value={perfMetrics.cumulative_layout_shift}
                                        metric="cumulative_layout_shift"
                                        icon={LayoutPanelTop}
                                    />
                                </div>
                            </CardContent>
                        </CollapsibleContent>
                    </Card>
                </Collapsible>
            )}

            {/* SEO Audits Breakdown */}
            {seoAudits && (
                <Collapsible open={seoOpen} onOpenChange={setSeoOpen}>
                    <Card>
                        <CollapsibleTrigger asChild>
                            <CardHeader className="pb-2 cursor-pointer hover:bg-muted/30 transition-colors">
                                <CardTitle className="flex items-center justify-between text-base">
                                    <div className="flex items-center gap-2">
                                        <Search className="h-4 w-4" />
                                        SEO Audit Breakdown
                                    </div>
                                    <ChevronDown className={cn(
                                        "h-4 w-4 transition-transform",
                                        seoOpen && "rotate-180"
                                    )} />
                                </CardTitle>
                            </CardHeader>
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                            <CardContent className="pt-0">
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                                    <AuditBadge audit={seoAudits.document_title} label="Document Title" />
                                    <AuditBadge audit={seoAudits.meta_description} label="Meta Description" />
                                    <AuditBadge audit={seoAudits.viewport} label="Viewport Meta Tag" />
                                    <AuditBadge audit={seoAudits.canonical} label="Canonical URL" />
                                    <AuditBadge audit={seoAudits.hreflang} label="Hreflang Tags" />
                                    <AuditBadge audit={seoAudits.robots_txt} label="Robots.txt Valid" />
                                    <AuditBadge audit={seoAudits.crawlable_anchors} label="Crawlable Anchors" />
                                    <AuditBadge audit={seoAudits.link_text} label="Descriptive Link Text" />
                                    <AuditBadge audit={seoAudits.image_alt} label="Image Alt Attributes" />
                                    <AuditBadge audit={seoAudits.http_status_code} label="HTTP Status Code" />
                                    <AuditBadge audit={seoAudits.is_crawlable} label="Page is Crawlable" />
                                </div>
                            </CardContent>
                        </CollapsibleContent>
                    </Card>
                </Collapsible>
            )}
        </div>
    )
}

// Compact version for the page detail modal
export function LighthouseScoresCompact({ page }: { page: PageAnalysisData }) {
    if (!page.lighthouse_performance) return null

    const scores = [
        { label: "Performance", value: page.lighthouse_performance, icon: Zap },
        { label: "Accessibility", value: page.lighthouse_accessibility, icon: Eye },
        { label: "Best Practices", value: page.lighthouse_best_practices, icon: Shield },
        { label: "SEO", value: page.lighthouse_seo, icon: Search },
    ]

    return (
        <div className="grid grid-cols-4 gap-2">
            {scores.map((score) => (
                <div key={score.label} className="flex flex-col items-center gap-1 p-2 rounded-lg bg-muted/50">
                    <ScoreRing score={score.value || 0} size="sm" />
                    <span className="text-[10px] text-muted-foreground text-center">{score.label}</span>
                </div>
            ))}
        </div>
    )
}
