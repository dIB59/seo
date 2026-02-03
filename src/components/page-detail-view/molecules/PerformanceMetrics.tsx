import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import { Badge } from "@/src/components/ui/badge"
import type { LighthousePerformanceMetrics } from "@/src/lib/types"

function formatTime(ms: number | null): string {
    if (ms === null) return "N/A"
    if (ms < 1000) return `${Math.round(ms)}ms`
    return `${(ms / 1000).toFixed(1)}s`
}

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

export default function PerformanceMetrics({ metrics }: { metrics: LighthousePerformanceMetrics }) {
    const metricItems = [
        { key: "first_contentful_paint", label: "First Contentful Paint (FCP)", value: metrics.first_contentful_paint },
        { key: "largest_contentful_paint", label: "Largest Contentful Paint (LCP)", value: metrics.largest_contentful_paint },
        { key: "speed_index", label: "Speed Index", value: metrics.speed_index },
        { key: "time_to_interactive", label: "Time to Interactive (TTI)", value: metrics.time_to_interactive },
        { key: "total_blocking_time", label: "Total Blocking Time (TBT)", value: metrics.total_blocking_time },
        { key: "cumulative_layout_shift", label: "Cumulative Layout Shift (CLS)", value: metrics.cumulative_layout_shift },
    ]

    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Metric</TableHead>
                    <TableHead className="text-right">Value</TableHead>
                    <TableHead className="w-[100px] text-right">Status</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {metricItems.map(({ key, label, value }) => {
                    const isCLS = key === "cumulative_layout_shift"
                    const displayValue = isCLS ? (value?.toFixed(3) ?? "N/A") : formatTime(value)
                    const colorClass = getMetricColor(key, value)

                    return (
                        <TableRow key={key}>
                            <TableCell className="font-medium">{label}</TableCell>
                            <TableCell className={`${colorClass} text-right font-mono`}>{displayValue}</TableCell>
                            <TableCell className="text-right">
                                <Badge variant="outline" className={`text-xs ${colorClass === "text-success" ? "bg-success/15 text-success border-success/20" : colorClass === "text-warning" ? "bg-warning/15 text-warning border-warning/20" : colorClass === "text-destructive" ? "bg-destructive/15 text-destructive border-destructive/20" : ""}`}>
                                    {colorClass === "text-success" ? "Good" : colorClass === "text-warning" ? "Needs Work" : colorClass === "text-destructive" ? "Poor" : "N/A"}
                                </Badge>
                            </TableCell>
                        </TableRow>
                    )
                })}
            </TableBody>
        </Table>
    )
}
