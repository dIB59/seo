import React from "react"

export function useAnalysisProgress(jobId: string, jobStatus: string) {
    const [pagesAnalyzed, setPagesAnalyzed] = React.useState<number | null>(null)

    React.useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            const { listen } = await import("@tauri-apps/api/event")
            unlisten = await listen<{ job_id: string; progress: number; pages_analyzed: number }>(
                "analysis:progress",
                (event) => {
                    if (event.payload.job_id === jobId) {
                        setPagesAnalyzed(event.payload.pages_analyzed ?? null)
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

    return pagesAnalyzed
}
