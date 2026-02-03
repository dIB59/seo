import { XCircle, AlertCircle, Lightbulb, AlertTriangle } from "lucide-react"

export function IssueIcon({ type }: { type: string }) {
    const iconMap: Record<string, React.ReactNode> = {
        Critical: <XCircle className="h-4 w-4 text-destructive" />,
        Warning: <AlertCircle className="h-4 w-4 text-warning" />,
        Suggestion: <Lightbulb className="h-4 w-4 text-primary" />,
    }
    return iconMap[type] ?? <AlertTriangle className="h-4 w-4 text-muted-foreground" />
}
