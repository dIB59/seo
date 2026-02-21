import { JobStatus } from "@/src/bindings"
import { CheckCircle2, Clock, Loader2, Search, XCircle, Ban } from "lucide-react"

export function getStatusIcon(status: JobStatus) {
    switch (status) {
        case "pending":
            return <Clock className="h-4 w-4 text-muted-foreground" />
        case 'discovery':
            return <Search className="h-4 w-4 text-primary animate-pulse" />
        case "processing":
            return <Loader2 className="h-4 w-4 text-primary animate-spin" />
        case "completed":
            return <span className="text-success"><CheckCircle2 className="h-4 w-4" /></span>
        case "failed":
            return <XCircle className="h-4 w-4 text-destructive" />
        case "cancelled":
            return <Ban className="h-4 w-4 text-muted-foreground/60" />
        default:
            return <Clock className="h-4 w-4 text-muted-foreground" />
    }
}