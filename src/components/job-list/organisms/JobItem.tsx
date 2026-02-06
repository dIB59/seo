import type { AnalysisProgress } from "@/src/lib/types"
import { DiscoveryProgress } from "../atoms/DiscoveryProgress"
import { getStatusIcon } from "../atoms/JobStatusIcon"
import { CancelButton } from "../atoms/CancelButton"
import { ViewResultButton } from "../atoms/ViewResultButton"
import { JobProgressBar } from "../atoms/JobProgressBar"

import { useDiscoveryProgress } from "../hooks/useDiscoveryProgress"
import { JobItemHeader } from "../molecules/JobItemHeader"

interface JobItemProps {
    job: AnalysisProgress
    onViewResult: (jobId: string) => void
    onCancel: (jobId: string) => void
}

export function JobItem({ job, onViewResult, onCancel }: JobItemProps) {
    const discovery = useDiscoveryProgress(job.job_id, job.job_status)

    return (
        <div
            className="group flex items-center gap-4 p-4 bg-card border border-border rounded-lg hover:border-primary/50 transition-colors"
        >
            <div className="flex-shrink-0">{getStatusIcon(job.job_status)}</div>

            <div className="flex-1 min-w-0">
                <JobItemHeader job={job} />

                {(job.job_status === "processing" || job.job_status === "running") && job.progress !== null && (
                    <JobProgressBar
                        progress={job.progress}
                        current={discovery.count ?? job.analyzed_pages}
                        total={discovery.total ?? job.total_pages}
                    />
                )}

                {job.job_status === "discovering" && (
                    <DiscoveryProgress count={discovery.count} total={discovery.total} />
                )}
            </div>

            <div className="flex items-center gap-2">
                {job.job_status === "completed" && (
                    <ViewResultButton onClick={() => onViewResult(job.job_id)} />
                )}
                {(job.job_status === "queued" || job.job_status === "processing" || job.job_status === "discovering") && (
                    <CancelButton onClick={() => onCancel(job.job_id)} />
                )}
            </div>
        </div>
    )
}
