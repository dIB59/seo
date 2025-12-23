import React from "react"
import { Clock, CheckCircle2, XCircle, Loader2, ExternalLink, X, Search } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Progress } from "@/src/components/ui/progress"
import type { AnalysisProgress } from "@/src/lib/types"
import { JobStatusBadge } from "../atoms/JobStatusBadge"
import { DiscoveryProgress } from "../molecules/DiscoveryProgress"

interface JobItemProps {
    job: AnalysisProgress
    onViewResult: (jobId: number) => void
    onCancel: (jobId: number) => void
}

function getStatusIcon(status: string) {
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

export function JobItem({ job, onViewResult, onCancel }: JobItemProps) {
    const [discoveryCount, setDiscoveryCount] = React.useState<number | null>(null)

    React.useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            const { listen } = await import("@tauri-apps/api/event")
            unlisten = await listen<{ job_id: number; count: number }>(
                "discovery-progress",
                (event) => {
                    if (event.payload.job_id === job.job_id) {
                        setDiscoveryCount(event.payload.count)
                    }
                }
            )
        }

        if (job.job_status === "discovering") {
            setupListener()
        }

        return () => {
            if (unlisten) unlisten()
        }
    }, [job.job_id, job.job_status])

    return (
        <div
            className="group flex items-center gap-4 p-4 bg-card border border-border rounded-lg hover:border-primary/50 transition-colors"
        >
            <div className="flex-shrink-0">{getStatusIcon(job.job_status)}</div>

            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                    <span className="font-medium truncate">{job.url}</span>
                    <JobStatusBadge status={job.job_status} />
                </div>

                {job.job_status === "processing" && job.progress !== null && (
                    <div className="flex items-center gap-3 mt-2">
                        <Progress value={job.progress} className="flex-1 h-1.5" />
                        <span className="text-xs text-muted-foreground whitespace-nowrap">
                            {job.analyzed_pages ?? 0} / {job.total_pages ?? "?"} pages
                        </span>
                    </div>
                )}

                {job.job_status === "discovering" && (
                    <DiscoveryProgress count={discoveryCount} />
                )}
            </div>

            <div className="flex items-center gap-2">
                {job.job_status === "completed" && (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onViewResult(job.job_id)}
                        className="text-primary hover:text-primary"
                    >
                        <ExternalLink className="h-4 w-4 mr-1" />
                        View Results
                    </Button>
                )}
                {(job.job_status === "queued" || job.job_status === "processing" || job.job_status === "discovering") && (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onCancel(job.job_id)}
                        className="text-destructive hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                        <X className="h-4 w-4" />
                    </Button>
                )}
            </div>
        </div>
    )
}
