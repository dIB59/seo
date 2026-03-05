import useSWR from "swr"
import { commands, type AnalysisSettingsRequest } from "@/src/bindings"

const fetchDefaults = async (isFreeUser: boolean) => {
    const result = isFreeUser
        ? await commands.getFreeTierDefaults()
        : await commands.getAnalysisDefaults()

    if (result.status === "error") {
        throw new Error(result.error)
    }

    return result.data
}

export function useAnalysisDefaults(isFreeUser: boolean) {
    const { data, error, isLoading } = useSWR<AnalysisSettingsRequest>(
        ["analysis-defaults", isFreeUser],
        () => fetchDefaults(isFreeUser),
        {
            // Defaults only change if the user tier changes, so we disable all automatic revalidations
            revalidateIfStale: false,
            revalidateOnFocus: false,
            revalidateOnReconnect: false,
            dedupingInterval: 60000,
        }
    )

    return {
        defaults: data ?? null,
        isLoading,
        error: error ? (error.message || "Failed to fetch defaults") : null
    }
}
