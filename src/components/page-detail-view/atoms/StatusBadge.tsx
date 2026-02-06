import { Badge } from "@/src/components/ui/badge"
import { CheckCircle2, AlertCircle } from "lucide-react"

export default function StatusBadge({ hasContent, label }: { hasContent: boolean; label: string }) {
    return (
        <Badge
            variant="outline"
            className={hasContent ? "bg-success/15 text-success border-success/20 text-xs" : "bg-destructive/15 text-destructive border-destructive/20 text-xs"}
        >
            {hasContent ? <CheckCircle2 className="h-3 w-3 mr-1" /> : <AlertCircle className="h-3 w-3 mr-1" />}
            {label}
        </Badge>
    )
}
