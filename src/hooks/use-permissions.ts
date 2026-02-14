import { useState, useEffect } from "react";
import { getUserPolicy } from "@/src/api/permissions";
import type { Policy, Feature } from "@/src/bindings";
import { toast } from "@/src/hooks/use-toast";

export function usePermissions() {
    const [policy, setPolicy] = useState<Policy | undefined>(undefined);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        let mounted = true;

        const fetchPolicy = async () => {
            try {
                const result = await getUserPolicy();
                if (!mounted) return;

                if (result.isOk()) {
                    setPolicy(result.unwrap());
                } else {
                    const error = result.unwrapErr();
                    console.error("Failed to fetch policy:", error);
                    toast({
                        variant: "destructive",
                        title: "Permission Error",
                        description: "Could not load user permissions. Some features may be restricted.",
                    });
                }
            } catch (error) {
                console.error("Error fetching policy:", error);
            } finally {
                if (mounted) setIsLoading(false);
            }
        };

        fetchPolicy();

        return () => {
            mounted = false;
        };
    }, []);

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
        isLoading,
        hasFeature,
        canAnalyzePages,
        isFreeUser: policy?.tier === "Free",
        isPremiumUser: policy?.tier === "Premium",
        maxPages: policy?.max_pages ?? 1,
    };
}
