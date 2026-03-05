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
            <div className="flex flex-col items-center justify-center py-16 text-muted-foreground/60 border border-dashed border-border/50 rounded-xl bg-card/20">
                <div className="w-12 h-12 mb-4 rounded-full bg-muted/50 flex items-center justify-center">
                    <svg className="w-6 h-6 opacity-40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                    </svg>
                </div>
                <p className="text-sm font-medium">No analysis jobs found</p>
                <p className="text-xs mt-1">Submit a URL above to start a new analysis</p>
            </div>
        )
    }

    return (
        <div className="rounded-xl border border-border/60 bg-card/30 overflow-hidden shadow-sm">
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
