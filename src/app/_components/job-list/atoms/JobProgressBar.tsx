import { Progress } from "@/src/components/ui/progress"

interface JobProgressBarProps {
    current: number
    total?: number | null
    label?: string
}

export function JobProgressBar({ current, total, label }: JobProgressBarProps) {
    const hasTotal = !!(total && total > 0)
    const progressValue = hasTotal ? Math.max(0, Math.min(100, (current / (total as number)) * 100)) : 0

    return (
        <div className="mt-2 flex items-center gap-3">
            {hasTotal ? (
                <Progress value={progressValue} className="flex-1 h-1.5" />
            ) : (
                <div className="flex-1 overflow-hidden h-1.5 bg-muted rounded-full relative">
                    <div className="h-full bg-blue-500 animate-[shimmer_2s_infinite] w-[40%]" />
                </div>
            )}
            <span className="text-xs text-muted-foreground whitespace-nowrap min-w-[80px] text-right">
                {label || `${current} / ${total ?? "?"} pages`}
            </span>
        </div>
    )
}
