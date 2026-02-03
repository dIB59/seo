"use client"

import { useEffect, useCallback, useState } from "react"
import {
    ArrowLeft,
    ChevronLeft,
    ChevronRight,
    FileText,
    Clock,
    Link2,
    ImageIcon,
    Heading,
    ExternalLink,
    AlertCircle,
    CheckCircle2,
    Globe,
    Hash,
    Search,
    ChevronsUpDown,
    Gauge,
    Zap,
    Eye,
    Shield,
    Activity,
    LayoutPanelTop,
    MousePointer,
    XCircle,
} from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs"
import { Badge } from "@/src/components/ui/badge"
import { Separator } from "@/src/components/ui/separator"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
} from "@/src/components/ui/command"
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/src/components/ui/popover"
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/src/components/ui/tooltip"
import { cn } from "@/src/lib/utils"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/src/components/ui/dialog"
import type { PageDetailData, HeadingElement, ImageElement, LinkElement } from "@/src/lib/types"
import { ScoreRing } from "./analysis/atoms/ScoreRing"

// ============================================================================
// UTILITY COMPONENTS
// ============================================================================

function CharLengthBadge({ length, maxRecommended }: { length: number; maxRecommended?: number }) {
    const isWarning = maxRecommended && length > maxRecommended
    return (
        <Badge
            variant="outline"
            className={cn(
                "text-xs font-mono",
                isWarning ? "bg-warning/15 text-warning border-warning/20" : "bg-muted"
            )}
        >
            {length} chars
        </Badge>
    )
}

function StatusBadge({ hasContent, label }: { hasContent: boolean; label: string }) {
    return (
        <Badge
            variant="outline"
            className={cn(
                "text-xs",
                hasContent
                    ? "bg-success/15 text-success border-success/20"
                    : "bg-destructive/15 text-destructive border-destructive/20"
            )}
        >
            {hasContent ? <CheckCircle2 className="h-3 w-3 mr-1" /> : <AlertCircle className="h-3 w-3 mr-1" />}
            {label}
        </Badge>
    )
}

function getLoadTimeColor(time: number) {
    if (time < 1) return "text-success"
    if (time < 2) return "text-warning"
    return "text-destructive"
}

// ============================================================================
// TAB COMPONENTS
// ============================================================================

function MetaTab({ page }: { page: PageDetailData }) {
    const metaFields = [
        {
            label: "Title",
            value: page.title,
            maxLength: 60,
            icon: FileText,
        },
        {
            label: "Meta Description",
            value: page.meta_description,
            maxLength: 160,
            icon: FileText,
        },
        {
            label: "Meta Keywords",
            value: page.meta_keywords,
            icon: Hash,
        },
        {
            label: "Canonical URL",
            value: page.canonical_url,
            icon: Link2,
        },
    ]

    return (
        <Card>
            <CardContent className="pt-6">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[150px]">Field</TableHead>
                            <TableHead>Content</TableHead>
                            <TableHead className="w-[100px] text-right">Length</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {metaFields.map(({ label, value, maxLength, icon: Icon }) => (
                            <TableRow key={label}>
                                <TableCell className="font-medium">
                                    <div className="flex items-center gap-2">
                                        <Icon className="h-4 w-4 text-muted-foreground" />
                                        {label}
                                    </div>
                                </TableCell>
                                <TableCell>
                                    {value ? (
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <span className="text-sm truncate block max-w-[400px] cursor-default">
                                                        {value}
                                                    </span>
                                                </TooltipTrigger>
                                                <TooltipContent className="max-w-md">
                                                    <p className="break-words">{value}</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    ) : (
                                        <span className="text-muted-foreground italic">Not set</span>
                                    )}
                                </TableCell>
                                <TableCell className="text-right">
                                    {value ? (
                                        <CharLengthBadge length={value.length} maxRecommended={maxLength} />
                                    ) : (
                                        <span className="text-muted-foreground">-</span>
                                    )}
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
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

function SeoAuditTab({ page }: { page: PageDetailData }) {
    const seoAudits = page.lighthouse_seo_audits
    const perfMetrics = page.lighthouse_performance_metrics
    
    // No SEO data available at all
    if (!page.lighthouse_seo) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <Search className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No SEO audit data available</p>
                    <p className="text-sm text-muted-foreground mt-1">
                        Run an analysis to see SEO scores
                    </p>
                </CardContent>
            </Card>
        )
    }

    const scores = [
        { label: "Performance", value: page.lighthouse_performance, icon: Zap, color: "text-orange-500" },
        { label: "Accessibility", value: page.lighthouse_accessibility, icon: Eye, color: "text-blue-500" },
        { label: "Best Practices", value: page.lighthouse_best_practices, icon: Shield, color: "text-purple-500" },
        { label: "SEO", value: page.lighthouse_seo, icon: Search, color: "text-green-500" },
    ]

    const auditItems = seoAudits ? [
        { key: "document_title", label: "Document Title", audit: seoAudits.document_title },
        { key: "meta_description", label: "Meta Description", audit: seoAudits.meta_description },
        { key: "viewport", label: "Viewport Meta Tag", audit: seoAudits.viewport },
        { key: "canonical", label: "Canonical URL", audit: seoAudits.canonical },
        { key: "hreflang", label: "Hreflang Tags", audit: seoAudits.hreflang },
        { key: "robots_txt", label: "Robots.txt Valid", audit: seoAudits.robots_txt },
        { key: "crawlable_anchors", label: "Crawlable Anchors", audit: seoAudits.crawlable_anchors },
        { key: "link_text", label: "Descriptive Link Text", audit: seoAudits.link_text },
        { key: "image_alt", label: "Image Alt Attributes", audit: seoAudits.image_alt },
        { key: "http_status_code", label: "HTTP Status Code", audit: seoAudits.http_status_code },
        { key: "is_crawlable", label: "Page is Crawlable", audit: seoAudits.is_crawlable },
    ] : []

    const metricItems = perfMetrics ? [
        { key: "first_contentful_paint", label: "First Contentful Paint (FCP)", value: perfMetrics.first_contentful_paint, icon: Clock },
        { key: "largest_contentful_paint", label: "Largest Contentful Paint (LCP)", value: perfMetrics.largest_contentful_paint, icon: LayoutPanelTop },
        { key: "speed_index", label: "Speed Index", value: perfMetrics.speed_index, icon: Gauge },
        { key: "time_to_interactive", label: "Time to Interactive (TTI)", value: perfMetrics.time_to_interactive, icon: MousePointer },
        { key: "total_blocking_time", label: "Total Blocking Time (TBT)", value: perfMetrics.total_blocking_time, icon: Clock },
        { key: "cumulative_layout_shift", label: "Cumulative Layout Shift (CLS)", value: perfMetrics.cumulative_layout_shift, icon: LayoutPanelTop },
    ] : []

    return (
        <div className="space-y-4">
            {/* Score Overview */}
            <Card>
                <CardHeader className="pb-2">
                    <CardTitle className="flex items-center gap-2 text-base">
                        <Search className="h-4 w-4" />
                        {perfMetrics ? "Deep Audit Scores" : "SEO Scores"}
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="grid grid-cols-4 gap-4">
                        {scores.map((score) => {
                            const Icon = score.icon
                            return (
                                <div
                                    key={score.label}
                                    className="flex flex-col items-center gap-2 p-4 rounded-lg bg-muted/50"
                                >
                                    <ScoreRing score={score.value || 0} size="md" />
                                    <div className="flex items-center gap-1.5">
                                        <Icon className={cn("h-4 w-4", score.color)} />
                                        <span className="text-sm font-medium">{score.label}</span>
                                    </div>
                                </div>
                            )
                        })}
                    </div>
                </CardContent>
            </Card>

            {/* Performance Metrics */}
            {perfMetrics && (
                <Card>
                    <CardHeader className="pb-2">
                        <CardTitle className="flex items-center gap-2 text-base">
                            <Activity className="h-4 w-4" />
                            Core Web Vitals & Performance Metrics
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>Metric</TableHead>
                                    <TableHead className="text-right">Value</TableHead>
                                    <TableHead className="w-[100px] text-right">Status</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {metricItems.map(({ key, label, value, icon: Icon }) => {
                                    const isCLS = key === "cumulative_layout_shift"
                                    const displayValue = isCLS ? (value?.toFixed(3) ?? "N/A") : formatTime(value)
                                    const colorClass = getMetricColor(key, value)
                                    
                                    return (
                                        <TableRow key={key}>
                                            <TableCell className="font-medium">
                                                <div className="flex items-center gap-2">
                                                    <Icon className="h-4 w-4 text-muted-foreground" />
                                                    {label}
                                                </div>
                                            </TableCell>
                                            <TableCell className={cn("text-right font-mono", colorClass)}>
                                                {displayValue}
                                            </TableCell>
                                            <TableCell className="text-right">
                                                <Badge 
                                                    variant="outline" 
                                                    className={cn(
                                                        "text-xs",
                                                        colorClass === "text-success" && "bg-success/15 text-success border-success/20",
                                                        colorClass === "text-warning" && "bg-warning/15 text-warning border-warning/20",
                                                        colorClass === "text-destructive" && "bg-destructive/15 text-destructive border-destructive/20",
                                                    )}
                                                >
                                                    {colorClass === "text-success" ? "Good" : 
                                                     colorClass === "text-warning" ? "Needs Work" : 
                                                     colorClass === "text-destructive" ? "Poor" : "N/A"}
                                                </Badge>
                                            </TableCell>
                                        </TableRow>
                                    )
                                })}
                            </TableBody>
                        </Table>
                    </CardContent>
                </Card>
            )}

            {/* SEO Audits */}
            {seoAudits && (
                <Card>
                    <CardHeader className="pb-2">
                        <CardTitle className="flex items-center gap-2 text-base">
                            <Search className="h-4 w-4" />
                            SEO Audit Breakdown
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>Audit</TableHead>
                                    <TableHead className="w-[100px] text-right">Status</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {auditItems.map(({ key, label, audit }) => (
                                    <TableRow key={key}>
                                        <TableCell className="font-medium">
                                            <div className="flex items-center gap-2">
                                                {audit.passed ? (
                                                    <CheckCircle2 className="h-4 w-4 text-success" />
                                                ) : (
                                                    <XCircle className="h-4 w-4 text-destructive" />
                                                )}
                                                {label}
                                            </div>
                                        </TableCell>
                                        <TableCell className="text-right">
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
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>
                    </CardContent>
                </Card>
            )}
        </div>
    )
}

function HeadingsTab({ headings }: { headings: HeadingElement[] }) {
    if (!headings || headings.length === 0) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <Heading className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No headings found on this page</p>
                    <p className="text-sm text-muted-foreground mt-1">
                        Backend needs to populate the headings array
                    </p>
                </CardContent>
            </Card>
        )
    }

    const tagColors: Record<string, string> = {
        h1: "bg-primary text-primary-foreground",
        h2: "bg-primary/80 text-primary-foreground",
        h3: "bg-primary/60 text-primary-foreground",
        h4: "bg-primary/40 text-primary-foreground",
        h5: "bg-primary/30",
        h6: "bg-primary/20",
    }

    return (
        <Card>
            <CardContent className="pt-6">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[80px]">Tag</TableHead>
                            <TableHead>Content</TableHead>
                            <TableHead className="w-[100px] text-right">Length</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {headings.map((heading, idx) => (
                            <TableRow key={idx}>
                                <TableCell>
                                    <Badge className={cn("font-mono uppercase", tagColors[heading.tag])}>
                                        {heading.tag}
                                    </Badge>
                                </TableCell>
                                <TableCell className="max-w-[400px]">
                                    {heading.text ? (
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <span className="text-sm truncate block cursor-default">
                                                        {heading.text}
                                                    </span>
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p className="max-w-md break-words">{heading.text}</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    ) : (
                                        <span className="text-muted-foreground italic">Empty</span>
                                    )}
                                </TableCell>
                                <TableCell className="text-right">
                                    <CharLengthBadge length={heading.text.length} />
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}

function ImagesTab({ images }: { images: ImageElement[] }) {
    if (!images || images.length === 0) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <ImageIcon className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No images found on this page</p>
                    <p className="text-sm text-muted-foreground mt-1">
                        Backend needs to populate the images array
                    </p>
                </CardContent>
            </Card>
        )
    }

    const withAlt = images.filter((img) => img.alt !== null && img.alt.length > 0).length
    const missingAlt = images.length - withAlt

    const [previewSrc, setPreviewSrc] = useState<string | null>(null)

    const isDataURI = (s: string) => s.startsWith("data:") || s.includes("base64,")
    const isLongSrc = (s: string) => s.length > 2000
    const shouldOfferPreview = (s: string) => isDataURI(s) || isLongSrc(s)
    const truncate = (s: string, n = 120) => (s.length > n ? `${s.slice(0, n)}...` : s)

    return (
        <>
            <Card>
                <CardHeader className="pb-3">
                    <div className="flex items-center gap-3">
                        <Badge variant="outline" className="bg-success/15 text-success border-success/20">
                            {withAlt} with alt
                        </Badge>
                        {missingAlt > 0 && (
                            <Badge variant="outline" className="bg-destructive/15 text-destructive border-destructive/20">
                                {missingAlt} missing alt
                            </Badge>
                        )}
                    </div>
                </CardHeader>
                <CardContent className="pt-0">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Source</TableHead>
                                <TableHead>Alt Text</TableHead>
                                <TableHead className="w-[100px] text-center">Status</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {images.map((image, idx) => (
                                <TableRow key={idx}>
                                    <TableCell className="max-w-[250px]">
                                        <div className="flex items-center gap-2">
                                            <TooltipProvider>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <span className="text-sm truncate block font-mono text-muted-foreground cursor-default">
                                                            {shouldOfferPreview(image.src) ? truncate(image.src, 120) : image.src}
                                                        </span>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p className="max-w-md break-all font-mono text-xs">{shouldOfferPreview(image.src) ? `${truncate(image.src, 600)} (truncated)` : image.src}</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>

                                            {shouldOfferPreview(image.src) && (
                                                <Button variant="ghost" size="sm" onClick={() => setPreviewSrc(image.src)} aria-label="Preview image">
                                                    <Eye className="h-3 w-3" />
                                                </Button>
                                            )}
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        {image.alt ? (
                                            <TooltipProvider>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <span className="text-sm truncate block max-w-[300px] cursor-default">
                                                            {image.alt}
                                                        </span>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p className="max-w-md break-words">{image.alt}</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>
                                        ) : (
                                            <span className="text-muted-foreground italic">Missing</span>
                                        )}
                                    </TableCell>
                                    <TableCell className="text-center">
                                        <StatusBadge hasContent={!!image.alt} label={image.alt ? "OK" : "Missing"} />
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>

            <Dialog open={!!previewSrc} onOpenChange={() => setPreviewSrc(null)}>
                <DialogContent className="max-w-3xl max-h-[90vh] overflow-auto">
                    <DialogHeader>
                        <DialogTitle>Image Preview</DialogTitle>
                    </DialogHeader>
                    <div className="p-4 flex justify-center">
                        {previewSrc && (
                            // Render on-demand only when user opened preview
                            <img src={previewSrc} alt="preview" className="max-w-full max-h-[70vh] object-contain" />
                        )}
                    </div>
                </DialogContent>
            </Dialog>
        </>
    )
}

function LinksTab({ links }: { links: LinkElement[] }) {
    if (!links || links.length === 0) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <Link2 className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No links found on this page</p>
                    <p className="text-sm text-muted-foreground mt-1">
                        Backend needs to populate the links array
                    </p>
                </CardContent>
            </Card>
        )
    }

    const internalLinks = links.filter((l) => l.is_internal).length
    const externalLinks = links.length - internalLinks

    return (
        <Card>
            <CardHeader className="pb-3">
                <div className="flex items-center gap-3">
                    <Badge variant="outline" className="bg-primary/15 text-primary border-primary/20">
                        <Globe className="h-3 w-3 mr-1" />
                        {internalLinks} internal
                    </Badge>
                    <Badge variant="outline" className="bg-muted">
                        <ExternalLink className="h-3 w-3 mr-1" />
                        {externalLinks} external
                    </Badge>
                </div>
            </CardHeader>
            <CardContent className="pt-0">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>URL</TableHead>
                            <TableHead>Anchor Text</TableHead>
                            <TableHead className="w-[80px] text-center">Type</TableHead>
                            <TableHead className="w-[80px] text-center">Status</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {links.map((link, idx) => (
                            <TableRow key={idx}>
                                <TableCell className="max-w-[250px]">
                                    <TooltipProvider>
                                        <Tooltip>
                                            <TooltipTrigger asChild>
                                                <span className="text-sm truncate block font-mono text-muted-foreground cursor-default">
                                                    {link.href}
                                                </span>
                                            </TooltipTrigger>
                                            <TooltipContent>
                                                <p className="max-w-md break-all font-mono text-xs">{link.href}</p>
                                            </TooltipContent>
                                        </Tooltip>
                                    </TooltipProvider>
                                </TableCell>
                                <TableCell>
                                    {link.text ? (
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <span className="text-sm truncate block max-w-[300px] cursor-default">
                                                        {link.text}
                                                    </span>
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p className="max-w-md break-words">{link.text}</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    ) : (
                                        <span className="text-muted-foreground italic">No text</span>
                                    )}
                                </TableCell>
                                <TableCell className="text-center">
                                    <Badge
                                        variant="outline"
                                        className={cn(
                                            "text-xs",
                                            link.is_internal
                                                ? "bg-primary/15 text-primary border-primary/20"
                                                : "bg-muted"
                                        )}
                                    >
                                        {link.is_internal ? "Int" : "Ext"}
                                    </Badge>
                                </TableCell>
                                <TableCell className="text-center">
                                    {link.status_code ? (
                                        <Badge
                                            variant="outline"
                                            className={cn(
                                                "text-xs font-mono",
                                                link.status_code >= 200 && link.status_code < 300
                                                    ? "bg-success/15 text-success border-success/20"
                                                    : link.status_code >= 400
                                                        ? "bg-destructive/15 text-destructive border-destructive/20"
                                                        : "bg-warning/15 text-warning border-warning/20"
                                            )}
                                        >
                                            {link.status_code}
                                        </Badge>
                                    ) : (
                                        <span className="text-muted-foreground">-</span>
                                    )}
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

interface PageDetailViewProps {
    page: PageDetailData
    pages: PageDetailData[]
    currentIndex: number
    onBack: () => void
    onNavigate: (index: number) => void
}

export function PageDetailView({
    page,
    pages,
    currentIndex,
    onBack,
    onNavigate,
}: PageDetailViewProps) {
    const [searchOpen, setSearchOpen] = useState(false)

    const canGoPrev = currentIndex > 0
    const canGoNext = currentIndex < pages.length - 1

    const goToPrev = useCallback(() => {
        if (canGoPrev) onNavigate(currentIndex - 1)
    }, [canGoPrev, currentIndex, onNavigate])

    const goToNext = useCallback(() => {
        if (canGoNext) onNavigate(currentIndex + 1)
    }, [canGoNext, currentIndex, onNavigate])

    // Helper to get short path from URL
    const getShortPath = (url: string) => {
        try {
            return new URL(url).pathname || "/"
        } catch {
            return url.replace(/^https?:\/\/[^/]+/, "") || "/"
        }
    }

    // Keyboard navigation
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            // Don't trigger if user is typing in an input
            if (e.target instanceof HTMLInputElement) return

            if (e.key === "ArrowLeft") goToPrev()
            if (e.key === "ArrowRight") goToNext()
            if (e.key === "Escape") onBack()
            // Ctrl/Cmd + K to open search
            if ((e.metaKey || e.ctrlKey) && e.key === "k") {
                e.preventDefault()
                setSearchOpen(true)
            }
        }
        window.addEventListener("keydown", handleKeyDown)
        return () => window.removeEventListener("keydown", handleKeyDown)
    }, [goToPrev, goToNext, onBack])

    return (
        <div className="space-y-4">
            {/* Header with navigation and search */}
            <div className="flex items-center justify-between gap-4">
                <Button variant="ghost" size="sm" onClick={onBack}>
                    <ArrowLeft className="h-4 w-4 mr-2" />
                    Back to Results
                </Button>

                {/* Page Selector with Search */}
                <div className="flex items-center gap-2 flex-1 justify-center max-w-md">
                    <Popover open={searchOpen} onOpenChange={setSearchOpen}>
                        <PopoverTrigger asChild>
                            <Button
                                variant="outline"
                                role="combobox"
                                aria-expanded={searchOpen}
                                className="w-full justify-between text-left font-normal"
                            >
                                <div className="flex items-center gap-2 truncate">
                                    <Search className="h-4 w-4 text-muted-foreground shrink-0" />
                                    <span className="truncate">
                                        {getShortPath(page.url)}
                                    </span>
                                </div>
                                <div className="flex items-center gap-1 shrink-0">
                                    <Badge variant="secondary" className="text-xs">
                                        {currentIndex + 1}/{pages.length}
                                    </Badge>
                                    <ChevronsUpDown className="h-4 w-4 opacity-50" />
                                </div>
                            </Button>
                        </PopoverTrigger>
                        <PopoverContent className="w-[400px] p-0" align="center">
                            <Command>
                                <CommandInput placeholder="Search pages by URL or title..." />
                                <CommandList>
                                    <CommandEmpty>No pages found.</CommandEmpty>
                                    <CommandGroup heading="Pages">
                                        {pages.map((p, idx) => (
                                            <CommandItem
                                                key={idx}
                                                value={`${p.url} ${p.title || ""}`}
                                                onSelect={() => {
                                                    onNavigate(idx)
                                                    setSearchOpen(false)
                                                }}
                                                className="cursor-pointer"
                                            >
                                                <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                                                    <span className="font-medium text-sm truncate">
                                                        {getShortPath(p.url)}
                                                    </span>
                                                    <span className="text-xs text-muted-foreground truncate">
                                                        {p.title || "No title"}
                                                    </span>
                                                </div>
                                                {idx === currentIndex && (
                                                    <CheckCircle2 className="h-4 w-4 text-primary shrink-0" />
                                                )}
                                            </CommandItem>
                                        ))}
                                    </CommandGroup>
                                </CommandList>
                            </Command>
                        </PopoverContent>
                    </Popover>
                </div>

                {/* Prev/Next buttons */}
                <div className="flex gap-1">
                    <Button
                        variant="outline"
                        size="icon"
                        onClick={goToPrev}
                        disabled={!canGoPrev}
                        className="h-8 w-8"
                        title="Previous page (←)"
                    >
                        <ChevronLeft className="h-4 w-4" />
                    </Button>
                    <Button
                        variant="outline"
                        size="icon"
                        onClick={goToNext}
                        disabled={!canGoNext}
                        className="h-8 w-8"
                        title="Next page (→)"
                    >
                        <ChevronRight className="h-4 w-4" />
                    </Button>
                </div>
            </div>

            {/* Page Info Header */}
            <Card>
                <CardContent className="py-4">
                    <div className="flex flex-col gap-2">
                        <div className="flex items-center gap-2">
                            <h2 className="text-lg font-semibold truncate">{page.url}</h2>
                            <a
                                href={page.url}
                                rel="noopener noreferrer"
                                className="shrink-0"
                                target="_blank"
                                onClick={(e) => {
                                    e.preventDefault()
                                    open(page.url)
                                }}
                            >
                                <ExternalLink className="h-4 w-4 text-muted-foreground hover:text-foreground" />
                            </a>
                        </div>
                        <p className="text-sm text-muted-foreground truncate">
                            {page.title || "No title"}
                        </p>
                        <Separator className="my-2" />
                        <div className="flex flex-wrap gap-4 text-sm">
                            <div className="flex items-center gap-1.5">
                                <Badge variant="outline" className="font-mono">
                                    {page.status_code || "N/A"}
                                </Badge>
                                <span className="text-muted-foreground">Status</span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <Clock className="h-4 w-4 text-muted-foreground" />
                                <span className={getLoadTimeColor(page.load_time)}>
                                    {page.load_time.toFixed(2)}s
                                </span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <FileText className="h-4 w-4 text-muted-foreground" />
                                <span>{page.word_count.toLocaleString()} words</span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <ImageIcon className="h-4 w-4 text-muted-foreground" />
                                <span>
                                    {page.image_count} images
                                    {page.images_without_alt > 0 && (
                                        <span className="text-destructive ml-1">
                                            ({page.images_without_alt} no alt)
                                        </span>
                                    )}
                                </span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <Link2 className="h-4 w-4 text-muted-foreground" />
                                <span>
                                    {page.internal_links} int / {page.external_links} ext
                                </span>
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>

            {/* Tabs */}
            <Tabs defaultValue="meta" className="space-y-4">
                <TabsList className="grid w-full grid-cols-5">
                    <TabsTrigger value="meta">
                        <FileText className="h-4 w-4 mr-2" />
                        Meta
                    </TabsTrigger>
                    <TabsTrigger value="seo-audit">
                        <Search className="h-4 w-4 mr-2" />
                        SEO Audit
                    </TabsTrigger>
                    <TabsTrigger value="headings">
                        <Heading className="h-4 w-4 mr-2" />
                        Headings
                        <Badge variant="secondary" className="ml-2 text-xs">
                            {page.h1_count + page.h2_count + page.h3_count}
                        </Badge>
                    </TabsTrigger>
                    <TabsTrigger value="images">
                        <ImageIcon className="h-4 w-4 mr-2" />
                        Images
                        <Badge variant="secondary" className="ml-2 text-xs">
                            {page.image_count}
                        </Badge>
                    </TabsTrigger>
                    <TabsTrigger value="links">
                        <Link2 className="h-4 w-4 mr-2" />
                        Links
                        <Badge variant="secondary" className="ml-2 text-xs">
                            {page.internal_links + page.external_links}
                        </Badge>
                    </TabsTrigger>
                </TabsList>

                <TabsContent value="meta">
                    <MetaTab page={page} />
                </TabsContent>

                <TabsContent value="seo-audit">
                    <SeoAuditTab page={page} />
                </TabsContent>

                <TabsContent value="headings">
                    <HeadingsTab headings={page.headings || []} />
                </TabsContent>

                <TabsContent value="images">
                    <ImagesTab images={page.images || []} />
                </TabsContent>

                <TabsContent value="links">
                    <LinksTab links={page.detailed_links || []} />
                </TabsContent>
            </Tabs>

            {/* Keyboard hints */}
            <div className="text-center text-xs text-muted-foreground">
                <kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px]">←</kbd>
                <kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px] ml-1">→</kbd>
                <span className="ml-2">Navigate pages</span>
                <span className="mx-3">·</span>
                <kbd className="px-1.5 py-0.5 bg-muted rounded text-[10px]">Esc</kbd>
                <span className="ml-2">Back to results</span>
            </div>
        </div>
    )
}
