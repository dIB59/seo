import React from "react"

export function useDiscoveryProgress(jobId: string, jobStatus: string) {
    const [discoveryCount, setDiscoveryCount] = React.useState<number | null>(null)

    React.useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            const { listen } = await import("@tauri-apps/api/event")
            unlisten = await listen<{ job_id: string; count: number }>(
                "discovery-progress",
                (event) => {
                    if (event.payload.job_id === jobId) {
                        setDiscoveryCount(event.payload.count)
                    }
                }
            )
        }

        if (jobStatus === "discovering") {
            setupListener()
        }

        return () => {
            if (unlisten) unlisten()
        }
    }, [jobId, jobStatus])

    return discoveryCount
}
