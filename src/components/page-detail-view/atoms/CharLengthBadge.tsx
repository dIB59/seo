import { Badge } from "@/src/components/ui/badge"
import { cn } from "@/src/lib/utils"

export default function CharLengthBadge({ length, maxRecommended }: { length: number; maxRecommended?: number }) {
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
