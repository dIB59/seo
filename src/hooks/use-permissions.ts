import useSWR from "swr";
import { getUserPolicy } from "@/src/api/permissions";
import { getMachineId } from "@/src/api/licensing";
import type { Feature } from "@/src/api/permissions";

async function fetchPermissions() {
  const [policyRes, machineRes] = await Promise.all([getUserPolicy(), getMachineId()]);

  return {
    policy: policyRes.isOk() ? policyRes.unwrap() : undefined,
    machineId: machineRes.isOk() ? machineRes.unwrap() : "",
  };
}

export function usePermissions() {
  const { data, isLoading, mutate } = useSWR("app-permissions", fetchPermissions, {
    revalidateOnFocus: false,
    revalidateOnReconnect: false,
  });

  const policy = data?.policy;
  const machineId = data?.machineId || "";

  const hasFeature = (feature: Feature): boolean => {
    if (!policy) return false;
    return policy.enabled_features.includes(feature);
  };

  const canAnalyzePages = (count: number): boolean => {
    if (!policy) return false;
    return count <= policy.max_pages;
  };

  return {
    policy,
    machineId,
    isLoading,
    mutate,
    hasFeature,
    canAnalyzePages,
    isFreeUser: policy?.tier === "Free",
    isPremiumUser: policy?.tier === "Premium",
    maxPages: policy?.max_pages ?? 1,
  };
}
