import type { AnalysisProgress } from "@/src/lib/types"
import { JobItem } from "./organisms/JobItem"

interface JobListProps {
    jobs: AnalysisProgress[]
    onViewResult: (jobId: string) => void
    onCancel: (jobId: string) => void
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
                <JobItem
                    key={job.job_id}
                    job={job}
                    onViewResult={onViewResult}
                    onCancel={onCancel}
                />
            ))}
        </div>
    )
}
