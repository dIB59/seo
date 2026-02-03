import React from "react"

// Listens only to discovery events. Returns { count, total } where each can be null
// if not available. The frontend expects discovery events to include both values.
export function useDiscoveryProgress(jobId: string, jobStatus: string) {
    const [count, setCount] = React.useState<number | null>(null)
    const [total, setTotal] = React.useState<number | null>(null)

    React.useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupDiscovery = async () => {
            const { listen } = await import("@tauri-apps/api/event")
            unlisten = await listen<{ job_id: string; count: number; total_pages?: number }>(
                "discovery-progress",
                (event) => {
                    if (event.payload.job_id === jobId) {
                        setCount(event.payload.count ?? null)
                        setTotal(event.payload.total_pages ?? null)
                    }
                }
            )
        }

        if (jobStatus === "discovering" || jobStatus === "processing" || jobStatus === "running") {
            // subscribe during discovery, processing, or running so UI can show current/total
            // even after discovery completes and processing begins
            setupDiscovery()
        }

        return () => {
            if (unlisten) unlisten()
            setCount(null)
            setTotal(null)
        }
    }, [jobId, jobStatus])

    return { count, total }
}
