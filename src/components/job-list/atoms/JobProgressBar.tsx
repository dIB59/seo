import { Progress } from "@/src/components/ui/progress"

export function JobProgressBar({ progress, current, total }: { progress: number; current?: number | null; total?: number | null }) {
    return (
        <div className="flex items-center gap-3 mt-2">
            <Progress value={progress} className="flex-1 h-1.5" />
            <span className="text-xs text-muted-foreground whitespace-nowrap">
                {current ?? 0} / {total ?? "?"} pages
            </span>
        </div>
    )
}
