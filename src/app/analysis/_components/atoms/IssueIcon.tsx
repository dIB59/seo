import { XCircle, AlertCircle, Lightbulb, AlertTriangle } from "lucide-react"

export function IssueIcon({ type }: { type: string }) {
    const iconMap: Record<string, React.ReactNode> = {
        critical: <XCircle className="h-4 w-4 text-destructive" />,
        warning: <AlertCircle className="h-4 w-4 text-warning" />,
        info: <Lightbulb className="h-4 w-4 text-primary" />,
    }
    return iconMap[type] ?? <AlertTriangle className="h-4 w-4 text-muted-foreground" />
} 
