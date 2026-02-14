import type { AnalysisProgress } from "@/src/lib/types"
import { getStatusIcon } from "../atoms/JobStatusIcon"
import { CancelButton } from "../atoms/CancelButton"
import { ViewResultButton } from "../atoms/ViewResultButton"
import { JobProgressBar } from "../atoms/JobProgressBar"

import { JobItemHeader } from "../molecules/JobItemHeader"
import { ScanSearch } from "lucide-react"
import { useDiscoveryProgress } from "../hooks/useDiscoveryProgress"
import { useAnalysisProgress } from "../hooks/useAnalysisProgress"

interface JobItemProps {
    job: AnalysisProgress
    onViewResult: (jobId: string) => void
    onCancel: (jobId: string) => void
}

export function JobItem({ job, onViewResult, onCancel }: JobItemProps) {
    const { count: pagesDiscovered, total: discoveryTotal } = useDiscoveryProgress(job.job_id, job.job_status)
    const pagesAnalyzed = useAnalysisProgress(job.job_id, job.job_status)

    const isDiscovering = job.job_status === "discovery"
    const isAnalyzing = job.job_status === "processing"

    return (
        <div
            className="group flex items-center gap-4 p-4 bg-card border border-border rounded-lg hover:border-primary/50 transition-colors"
        >
            <div className="flex-shrink-0 grid grid-cols-2 gap-2 place-items-center">
                <div className="flex justify-center">
                    {getStatusIcon(job.job_status)}
                </div>

                {job.total_issues !== undefined && job.total_issues !== null && (
                    <div className="flex justify-center" title={`Total Issues: ${job.total_issues}`}>
                        <div className="flex items-center justify-center w-6 h-6 bg-destructive/10 text-destructive rounded font-medium text-[10px]">
                            {job.total_issues > 99 ? '99+' : job.total_issues}
                        </div>
                    </div>
                )}

                {job.max_pages !== undefined && (
                    <div className="flex justify-center" title={`Page Limit: ${job.max_pages}`}>
                        <div className="flex items-center justify-center w-6 h-6 bg-muted text-[10px] font-medium text-muted-foreground rounded">
                            {job.max_pages}
                        </div>
                    </div>
                )}
                {job.is_deep_audit && (
                    <div className="flex justify-center" title="Deep Audit Enabled">
                        <div className="flex items-center justify-center w-6 h-6 bg-amber-500/10 text-amber-600 rounded">
                            <ScanSearch className="w-3.5 h-3.5" />
                        </div>
                    </div>
                )}
            </div>

            <div className="flex-1 min-w-0">
                <JobItemHeader job={job} />

                {isAnalyzing && pagesAnalyzed !== null && (
                    <JobProgressBar
                        current={pagesAnalyzed}
                        total={discoveryTotal}
                        label={`Analyzed ${pagesAnalyzed} / ${discoveryTotal}`}
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

            <div className="flex items-center gap-2 max-h-[30px]">
                {job.job_status === "completed" ? (
                    <ViewResultButton onClick={() => onViewResult(job.job_id)} />
                ) : (
                    <CancelButton onClick={() => onCancel(job.job_id)} />
                )}
            </div>
        </div>
    )
}
