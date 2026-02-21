"use client";

import { useCallback, useState } from "react";
import { mutate as globalMutate } from "swr";
import { HomeHeader } from "@/src/components/home/HomeHeader";
import { JobHistory } from "@/src/components/job-list/organisms/JobHistory";
import { startAnalysis } from "@/src/api/analysis";
import type { AnalysisSettingsRequest } from "@/src/lib/types";
import { logger } from "../lib/logger";
import { UrlInputForm } from "../components/url-input/UrlInputForm";

export default function Home() {
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleSubmit = useCallback(async (url: string, settings: AnalysisSettingsRequest) => {
        setIsLoading(true);
        setError(null);

        const res = await startAnalysis(url, settings);
        res.matchAsync(async () => {
            // Trigger refresh of all paginated job queries
            await globalMutate((key) => Array.isArray(key) && key[0] === "jobs-paginated");
            logger.info("Triggered global mutate for jobs-paginated");
        }, setError);

        setIsLoading(false);
    }, [setError]);

    const handleRefresh = useCallback(() => {
        globalMutate((key) => Array.isArray(key) && key[0] === "jobs-paginated");
    }, []);

    return (
        <main className="min-h-screen p-6 max-w-5xl mx-auto flex flex-col">
            <HomeHeader
                isValidating={false} // Validation status is now managed inside JobHistory
                onRefresh={handleRefresh}
            />

            {/* Error Message */}
            {error && (
                <div className="mb-6 p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
                    <p className="text-sm text-destructive">{error}</p>
                </div>
            )}

            {/* URL Input Form */}
            <div className="mb-2">
                <UrlInputForm onSubmit={handleSubmit} isLoading={isLoading} />
            </div>

            <JobHistory />
        </main>
    );
}
