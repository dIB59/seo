import type { AnalysisProgress } from "@/src/lib/types"
import { getStatusIcon } from "../atoms/JobStatusIcon"
import { CancelButton } from "../atoms/CancelButton"
import { ViewResultButton } from "../atoms/ViewResultButton"
import { JobProgressBar } from "../atoms/JobProgressBar"
import { useDiscoveryProgress } from "../hooks/useDiscoveryProgress"
import { useAnalysisProgress } from "../hooks/useAnalysisProgress"

interface JobItemProps {
    job: AnalysisProgress
    onViewResult: (jobId: string) => void
    onCancel: (jobId: string) => void
}

export function JobItem({ job, onViewResult, onCancel }: JobItemProps) {
    const { count: pagesDiscovered, total: discoveryTotal } = useDiscoveryProgress(job.job_id, job.job_status)
    const { current: pagesAnalyzed, total: analysisTotal } = useAnalysisProgress(job.job_id, job.job_status)

    const isDiscovering = job.job_status === "discovery"
    const isAnalyzing = job.job_status === "processing"
    const isCompleted = job.job_status === "completed"
    const canCancel = ["pending", "discovery", "processing"].includes(job.job_status)

    return (
        <div
            className="group relative flex items-center gap-4 p-3 bg-card/40 hover:bg-card/90 border-b border-border/50 first:border-t hover:border-transparent hover:shadow-sm transition-all duration-200"
        >
            {/* Status Column */}
            <div className="flex-shrink-0 w-10 flex justify-center">
                <div className="relative">
                    <div className="absolute inset-0 bg-primary/20 blur-md rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-500 scale-150" />
                    {getStatusIcon(job.job_status)}
                </div>
            </div>

            {/* Main Info */}
            <div className="flex-1 min-w-0 flex flex-col justify-center gap-1.5 py-1">
                {/* URL */}
                <div className="flex items-center gap-2">
                    <span className="font-medium text-sm truncate text-foreground/90 tracking-tight">{job.url}</span>
                </div>

                {/* Progress Bar */}
                {(isAnalyzing || isDiscovering) && (
                    <div className="max-w-[200px]">
                        {isAnalyzing && pagesAnalyzed !== null && (
                            <JobProgressBar
                                current={pagesAnalyzed}
                                total={analysisTotal ?? pagesDiscovered ?? 0}
                                label={`Analyzed ${pagesAnalyzed} / ${analysisTotal ?? pagesDiscovered ?? '?'}`}
                            />
                        )}
                        {isDiscovering && (
                            <JobProgressBar
                                current={pagesDiscovered ?? 0}
                                total={discoveryTotal}
                                label={!discoveryTotal ? `Discovered ${pagesDiscovered ?? 0} / ${discoveryTotal}` : undefined}
                            />
                        )}
                    </div>
                )}
            </div>

            {/* Metrics */}
            <div className="hidden lg:flex items-center gap-6 mr-6">
                {/* Pages Limit */}
                {job.max_pages !== undefined && (
                    <div className="flex flex-col items-end gap-1.5 w-12">
                        <span className="text-[10px] uppercase tracking-wider text-muted-foreground/50 font-medium leading-none">Limit</span>
                        <span className="text-xs font-mono text-muted-foreground/80 leading-none">{job.max_pages}</span>
                    </div>
                )}

                {/* Audit Mode */}
                <div className="flex flex-col items-end gap-1.5 w-16">
                    <span className="text-[10px] uppercase tracking-wider text-muted-foreground/50 font-medium leading-none">Auditor</span>
                    <span className="text-xs font-mono text-muted-foreground/80 leading-none">{job.is_deep_audit ? "DEEP" : "STANDARD"}</span>
                </div>

                {/* Issues Count */}
                {job.total_issues !== undefined && job.total_issues !== null && (
                    <div className="flex flex-col items-end gap-1.5 w-12">
                        <span className="text-[10px] uppercase tracking-wider text-muted-foreground/50 font-medium leading-none">Issues</span>
                        <div className={`flex items-center gap-1.5 text-xs font-mono leading-none ${job.total_issues > 0 ? "text-destructive" : "text-muted-foreground/40"}`}>
                            <span className={`w-1.5 h-1.5 rounded-full ${job.total_issues > 0 ? "bg-destructive/80" : "bg-muted-foreground/20"}`} />
                            {job.total_issues > 999 ? '999+' : job.total_issues}
                        </div>
                    </div>
                )}
            </div>

            {/* Actions */}
            <div className="flex-shrink-0 w-[140px] flex justify-end items-center border-l border-border/40 min-h-[44px]">
                <div className="flex items-center justify-end w-full gap-2">
                    {isCompleted && <ViewResultButton onClick={() => onViewResult(job.job_id)} />}
                    {canCancel && <CancelButton onClick={() => onCancel(job.job_id)} />}
                </div>
            </div>
        </div>
    )
}
