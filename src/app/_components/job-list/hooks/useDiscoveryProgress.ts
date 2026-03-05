import React from "react"
import type { ProgressEvent } from "@/src/bindings"
import { listen } from "@tauri-apps/api/event"

// Listens only to discovery events. Returns { count, total } where each can be null
// if not available. The frontend expects discovery events to include both values.
export function useDiscoveryProgress(jobId: string, jobStatus: string) {
    const [count, setCount] = React.useState<number | null>(null)
    const [total, setTotal] = React.useState<number | null>(null)

    React.useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupDiscovery = async () => {
            unlisten = await listen<ProgressEvent>(
                "discovery:progress",
                (event) => {
                    if (event.payload.job_id === jobId && event.payload.event === "discovery") {
                        setCount(event.payload.count ?? null)
                        setTotal(event.payload.total_pages ?? null)
                    }
                }
            )
        }

        if (jobStatus === "discovery" || jobStatus === "processing") {
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
