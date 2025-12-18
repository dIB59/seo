import { Progress } from "../../ui/progress"

export function ProgressRow({
    label,
    value,
    total,
    color,
}: {
    label: string
    value: number
    total: number
    color: "success" | "warning" | "destructive" | "primary"
}) {
    const colorMap = {
        success: "[&>div]:bg-success",
        warning: "[&>div]:bg-warning",
        destructive: "[&>div]:bg-destructive",
        primary: "[&>div]:bg-primary",
    }
    return (
        <div className="flex items-center gap-3">
            <div className="w-20 text-sm text-muted-foreground">{label}</div>
            <Progress value={total > 0 ? (value / total) * 100 : 0} className={cn("flex-1 h-2", colorMap[color])} />
            <div className="w-8 text-sm text-right">{value}</div>
        </div>
    )
}