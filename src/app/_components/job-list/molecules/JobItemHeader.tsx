import type { AnalysisProgress } from "@/src/api/analysis";
import { JobStatusBadge } from "../atoms/JobStatusBadge";

export function JobItemHeader({ job }: { job: AnalysisProgress }) {
  return (
    <div className="flex items-center justify-between gap-4 mb-2">
      <div className="flex items-center gap-3 min-w-0">
        <div className="w-1.5 h-1.5 rounded-full bg-primary/50" />
        <span className="font-mono text-sm font-medium truncate text-foreground/90 tracking-tight">
          {job.url}
        </span>
      </div>
      <JobStatusBadge status={job.job_status} />
    </div>
  );
}
