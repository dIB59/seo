"use client";

import { useState } from "react";
import useSWR from "swr";
import { useRouter } from "next/navigation";
import { Search, RefreshCw, Settings } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { UrlInputForm } from "@/src/components/url-input-form";
import { getAllJobs, startAnalysis, cancelAnalysis } from "@/src/api/analysis";
import type { AnalysisSettingsRequest } from "@/src/lib/types";
import { logger } from "../lib/logger";
import { JobList } from "@/src/components/job-list/JobList";

const fetchJobs = () =>
    getAllJobs().then((res) => {
        return res.unwrapOr([]);
    });

export default function Home() {
    const router = useRouter();
    const [isLoading, setIsLoading] = useState(false);
    // selectedResult state removed used for routing
    const [error, setError] = useState<string | null>(null);

    const {
        data: jobs = [],
        mutate,
        isValidating,
    } = useSWR("jobs", fetchJobs, {
        refreshInterval: 10_000,
        onError: (e) => setError(e instanceof Error ? e.message : String(e)),
    });

    const handleSubmit = async (url: string, settings: AnalysisSettingsRequest) => {
        setIsLoading(true);
        setError(null);

        const res = await startAnalysis(url, settings);
        res.matchAsync(async () => {
            await mutate();
            logger.info("Running Mutate");
        }, setError);

        setIsLoading(false);
    };

    const handleViewResult = async (jobId: number) => {
        router.push(`/analysis?id=${jobId}`);
    };
    // Note: using window.location.href or router.push if I import router.
    // Since "use client" is at top, I should import useRouter.

    const handleCancel = async (jobId: number) => {
        const res = await cancelAnalysis(jobId);
        res.match(
            () => {
                mutate();
            }, // void
            setError, // void
        );
    };

    return (
        <main className="min-h-screen p-6 max-w-5xl mx-auto">
            {/* Header */}
            <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-primary/20 rounded-lg">
                        <Search className="h-6 w-6 text-primary" />
                    </div>
                    <div>
                        <h1 className="text-2xl font-bold">SEO Analyzer</h1>
                        <p className="text-sm text-muted-foreground">
                            Analyze websites for SEO issues and recommendations
                        </p>
                    </div>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => mutate()}
                        disabled={isValidating}
                    >
                        <RefreshCw
                            className={`h-4 w-4 mr-2 ${isValidating ? "animate-spin" : ""}`}
                        />
                        Refresh
                    </Button>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() =>
                            window.dispatchEvent(new CustomEvent("open-settings-dialog"))
                        }
                    >
                        <Settings className="h-4 w-4 mr-2" />
                        AI Configuration
                    </Button>
                </div>
            </div>

            {/* Error Message */}
            {error && (
                <div className="mb-6 p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
                    <p className="text-sm text-destructive">{error}</p>
                </div>
            )}

            {/* URL Input Form */}
            <div className="mb-8">
                <UrlInputForm onSubmit={handleSubmit} isLoading={isLoading} />
            </div>

            {/* Analysis Jobs */}
            <div>
                <div className="flex items-center justify-between mb-4">
                    <h2 className="text-lg font-semibold">Analysis Jobs</h2>
                    {jobs.length > 0 && (
                        <span className="text-sm text-muted-foreground">{jobs.length} jobs</span>
                    )}
                </div>
                <JobList jobs={jobs} onViewResult={handleViewResult} onCancel={handleCancel} />
            </div>
        </main>
    );
}
