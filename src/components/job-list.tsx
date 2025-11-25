"use client"
import { Clock, CheckCircle2, XCircle, Loader2, ExternalLink, X } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Progress } from "@/src/components/ui/progress"
import { cn } from "@/src/lib/utils"
import type { AnalysisProgress } from "@/src/lib/types"

interface JobListProps {
  jobs: AnalysisProgress[]
  onViewResult: (jobId: number) => void
  onCancel: (jobId: number) => void
}

function getStatusIcon(status: string) {
  switch (status) {
    case "queued":
      return <Clock className="h-4 w-4 text-muted-foreground" />
    case "processing":
      return <Loader2 className="h-4 w-4 text-primary animate-spin" />
    case "completed":
      return <CheckCircle2 className="h-4 w-4 text-success" />
    case "failed":
      return <XCircle className="h-4 w-4 text-destructive" />
    default:
      return <Clock className="h-4 w-4 text-muted-foreground" />
  }
}

function getStatusBadge(status: string) {
  const baseClasses = "px-2 py-0.5 rounded-full text-xs font-medium"
  switch (status) {
    case "queued":
      return <span className={cn(baseClasses, "bg-muted text-muted-foreground")}>Queued</span>
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

export function JobList({ jobs, onViewResult, onCancel }: JobListProps) {
  if (jobs.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        <p>No analysis jobs yet. Submit a URL above to get started.</p>
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {jobs.map((job) => (
        <div
          key={job.job_id}
          className="group flex items-center gap-4 p-4 bg-card border border-border rounded-lg hover:border-primary/50 transition-colors"
        >
          <div className="flex-shrink-0">{getStatusIcon(job.job_status)}</div>

          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-medium truncate">{job.url}</span>
              {getStatusBadge(job.job_status)}
            </div>

            {job.job_status === "processing" && job.progress !== null && (
              <div className="flex items-center gap-3 mt-2">
								<Progress value={job.progress} className="flex-1 h-1.5" />
                <span className="text-xs text-muted-foreground whitespace-nowrap">
                  {job.analyzed_pages ?? 0} / {job.total_pages ?? "?"} pages
                </span>
              </div>
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
            {(job.job_status === "queued" || job.job_status === "processing") && (
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
      ))}
    </div>
  )
}
