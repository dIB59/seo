import { cn } from "@/src/lib/utils";
import { Badge } from "../../ui/badge";

export function IssueBadge({ type, count }: { type: string; count?: number }) {
    // Map backend severity to display label
    const displayLabel = type === "critical" ? "Critical" : type === "warning" ? "Warning" : "Suggestion"
    const label = count !== undefined ? `${count} ${displayLabel}` : displayLabel
    const variants: Record<string, string> = {
        critical: "bg-destructive/15 text-destructive border-destructive/20",
        warning: "bg-warning/15 text-warning border-warning/20",
        info: "bg-primary/15 text-primary border-primary/20",
    }
    return (
        <Badge variant="outline" className={cn("text-xs font-medium", variants[type] ?? "")}>{label}</Badge>
    )
} 
