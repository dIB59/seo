import { cn } from "@/src/lib/utils";
import { Badge } from "../../ui/badge";

export function IssueBadge({ type, count }: { type: string; count?: number }) {
    const label = count !== undefined ? `${count} ${type}` : type
    const variants: Record<string, string> = {
        Critical: "bg-destructive/15 text-destructive border-destructive/20",
        Warning: "bg-warning/15 text-warning border-warning/20",
        Suggestion: "bg-primary/15 text-primary border-primary/20",
    }
    return (
        <Badge variant="outline" className={cn("text-xs font-medium", variants[type])}>
            {label}
        </Badge>
    )
}
