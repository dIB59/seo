import { CheckCircle2, Clock, Loader2, Search, XCircle } from "lucide-react"

export function getStatusIcon(status: string) {
    switch (status) {
        case "queued":
            return <Clock className="h-4 w-4 text-muted-foreground" />
        case 'discovering':
            return <Search className="h-4 w-4 text-primary animate-pulse" />
        case "processing":
            return <Loader2 className="h-4 w-4 text-primary animate-spin" />
        case "completed":
            return <span className="text-success"><CheckCircle2 className="h-4 w-4" /></span>
        case "failed":
            return <XCircle className="h-4 w-4 text-destructive" />
        default:
            return <Clock className="h-4 w-4 text-muted-foreground" />
    }
}