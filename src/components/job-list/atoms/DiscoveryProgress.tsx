import { Progress } from "@/src/components/ui/progress"

export function DiscoveryProgress({ count, total }: { count: number | null; total?: number | null }) {
    if (total && total > 0) {
        const pct = Math.max(0, Math.min(100, Math.round(((count ?? 0) / total) * 100)))
        return (
            <div className="mt-2 flex items-center gap-3">
                <Progress value={pct} className="flex-1 h-1.5" />
                <span className="text-xs text-muted-foreground whitespace-nowrap">
                    {count ?? 0} / {total} pages • {pct}%
                </span>
            </div>
        )
    }

    // Indeterminate / discovery in progress (no total available yet)
    return (
        <div className="mt-2 flex items-center gap-2">
            <div className="flex-1 overflow-hidden h-1.5 bg-muted rounded-full">
                <div className="h-full bg-blue-500 animate-[shimmer_2s_infinite] w-[40%]" />
            </div>
            <span className="text-xs text-muted-foreground whitespace-nowrap">
                Discovered {count ?? 0} pages...
            </span>
        </div>
    )
}
