import useSWR from "swr";
import { getResult } from "@/src/api/analysis";
import { CompleteAnalysisResult } from "@/src/lib/types";

// Helper for SWR
const fetchAnalysis = async (id: string) => {
    const res = await getResult(id);
    return res.unwrap();
};

export function useAnalysis(id: string) {
    const { data, error, isLoading, mutate } = useSWR<CompleteAnalysisResult>(
        id ? `analysis-${id}` : null,
        () => fetchAnalysis(id),
        {
            // Don't auto revalidate too aggressively for static reports
            revalidateOnFocus: false,
        }
    );

    return {
        result: data,
        isLoading,
        isError: error,
        mutate,
    };
}
