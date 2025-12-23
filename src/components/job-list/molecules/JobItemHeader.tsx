import { AnalysisProgress } from "@/src/lib/types";
import { JobStatusBadge } from "../atoms/JobStatusBadge";

export function JobItemHeader({ job }: { job: AnalysisProgress }) {
    return (
        <div className="flex items-center gap-2 mb-1">
            <span className="font-medium truncate">{job.url}</span>
            <JobStatusBadge status={job.job_status} />
        </div>
    )
}

