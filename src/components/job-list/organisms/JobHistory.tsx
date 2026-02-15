"use client";

import { useState, useCallback } from "react";
import useSWR from "swr";
import { useRouter } from "next/navigation";
import { getPaginatedJobs, cancelAnalysis } from "@/src/api/analysis";
import { JobFilterBar } from "./JobFilterBar";
import { JobList } from "../JobList";
import { JobPagination } from "../molecules/JobPagination";

const fetchJobsPaginated = ([, limit, offset, urlFilter, statusFilter]: [string, number, number, string?, string?]) =>
    getPaginatedJobs(limit, offset, urlFilter, statusFilter).then((res) => {
        return res.unwrap();
    });

export function JobHistory() {
    const router = useRouter();
    const [error, setError] = useState<string | null>(null);

    // State
    const [currentPage, setCurrentPage] = useState(1);
    const [pageSize, setPageSize] = useState(5);
    const [urlFilter, setUrlFilter] = useState("");
    const [statusFilter, setStatusFilter] = useState("all");

    // Derived values
    const offset = (currentPage - 1) * pageSize;
    const s_urlFilter = urlFilter.trim() || undefined;
    const s_statusFilter = statusFilter === "all" ? undefined : statusFilter;

    // Data fetching
    const { data: paginatedData, mutate } = useSWR(
        ["jobs-paginated", pageSize, offset, s_urlFilter, s_statusFilter],
        fetchJobsPaginated,
        {
            refreshInterval: 5000,
            fallbackData: { items: [], total: 0 },
            onError: (err) => setError(err instanceof Error ? err.message : String(err)),
        }
    );

    const { items: jobs, total } = paginatedData;
    const totalPages = Math.ceil(total / pageSize);

    // Handlers
    const handleViewResult = useCallback((jobId: string) => {
        router.push(`/analysis?id=${jobId}`);
    }, [router]);

    const handleCancel = useCallback(async (jobId: string) => {
        const res = await cancelAnalysis(jobId);
        res.match(
            () => {
                mutate();
                setError(null);
            },
            setError,
        );
    }, [mutate]);

    return (
        <div className="flex-1 flex flex-col gap-6 relative min-h-[500px]">
            {/* Technical Background Pattern */}
            <div className="absolute inset-0 -z-10 opacity-[0.03]"
                style={{
                    backgroundImage: 'radial-gradient(#000 1px, transparent 1px)',
                    backgroundSize: '20px 20px'
                }}
            />

            <div className="flex flex-col gap-4">
                {/* Header / Filter Section */}
                <div className="backdrop-blur-sm bg-background/80 sticky top-0 z-10 pb-4 border-b border-border/40">
                    <JobFilterBar
                        total={total}
                        urlFilter={urlFilter}
                        setUrlFilter={setUrlFilter}
                        statusFilter={statusFilter}
                        setStatusFilter={setStatusFilter}
                        pageSize={pageSize}
                        setPageSize={setPageSize}
                        setCurrentPage={setCurrentPage}
                    />
                </div>

                {/* Error Message */}
                {error && (
                    <div className="p-4 bg-destructive/5 border border-destructive/20 rounded-md flex items-center gap-3 animate-in fade-in slide-in-from-top-2">
                        <div className="w-2 h-2 rounded-full bg-destructive animate-pulse" />
                        <p className="text-sm text-destructive font-mono">{error}</p>
                    </div>
                )}

                {/* Main Content Area */}
                <div className="relative">
                    <JobList
                        jobs={jobs}
                        onViewResult={handleViewResult}
                        onCancel={handleCancel}
                    />
                </div>
            </div>

            <JobPagination
                currentPage={currentPage}
                totalPages={totalPages}
                onPageChange={setCurrentPage}
                className="mt-auto pt-4 border-t border-border/40"
            />
        </div>
    );
}
