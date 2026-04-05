import useSWR from "swr";
import {
  getAnalysisDefaults,
  getFreeTierDefaults,
  type AnalysisSettingsRequest,
} from "@/src/api/analysis";

const fetchDefaults = async (isFreeUser: boolean) => {
  const result = isFreeUser ? await getFreeTierDefaults() : await getAnalysisDefaults();

  if (result.isErr()) {
    throw new Error(result.unwrapErr());
  }

  return result.unwrap();
};

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
    },
  );

  return {
    defaults: data ?? null,
    isLoading,
    error: error ? error.message || "Failed to fetch defaults" : null,
  };
}
