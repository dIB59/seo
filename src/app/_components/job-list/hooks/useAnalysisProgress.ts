import React from "react"
import type { ProgressEvent } from "@/src/bindings"
import { listen } from "@tauri-apps/api/event"

export function useAnalysisProgress(jobId: string, jobStatus: string) {
    const [pagesAnalyzed, setPagesAnalyzed] = React.useState<number | null>(null)
    const [totalPages, setTotalPages] = React.useState<number | null>(null)

    React.useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            unlisten = await listen<ProgressEvent>(
                "analysis:progress",
                (event) => {
                    if (event.payload.job_id === jobId && event.payload.event === "analysis") {
                        setPagesAnalyzed(event.payload.pages_analyzed ?? null)
                        setTotalPages(event.payload.total_pages ?? null)
                    }
                }
            )
        }

        if (jobStatus === "processing") {
            setupListener()
        }

        return () => {
            if (unlisten) unlisten()
        }
    }, [jobId, jobStatus])

    return { current: pagesAnalyzed, total: totalPages }
}
