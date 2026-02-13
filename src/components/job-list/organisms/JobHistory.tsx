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
        <div className="space-y-6">
            {/* Error Message */}
            {error && (
                <div className="p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
                    <p className="text-sm text-destructive">{error}</p>
                </div>
            )}

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

            <JobList
                jobs={jobs}
                onViewResult={handleViewResult}
                onCancel={handleCancel}
            />

            <JobPagination
                currentPage={currentPage}
                totalPages={totalPages}
                onPageChange={setCurrentPage}
            />
        </div>
    );
}
