import { cn } from "@/src/lib/utils"

export function JobStatusBadge({ status }: { status: string }) {
    const baseClasses = "px-2 py-0.5 rounded-full text-xs font-medium"
    switch (status) {
        case "queued":
            return <span className={cn(baseClasses, "bg-muted text-muted-foreground")}>Queued</span>
        case 'discovering':
            return <span className={cn(baseClasses, "bg-primary/20 text-primary")}>Discovering</span>
        case "processing":
            return <span className={cn(baseClasses, "bg-primary/20 text-primary")}>Processing</span>
        case "completed":
            return <span className={cn(baseClasses, "bg-success/20 text-success")}>Completed</span>
        case "failed":
            return <span className={cn(baseClasses, "bg-destructive/20 text-destructive")}>Failed</span>
        default:
            return <span className={cn(baseClasses, "bg-muted text-muted-foreground")}>{status}</span>
    }
}
