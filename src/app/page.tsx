"use client";

import { useState } from "react";
import useSWR from "swr";
import { useRouter } from "next/navigation";
import { Search, RefreshCw, Settings } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { UrlInputForm } from "@/src/components/url-input-form";
import { JobList } from "@/src/components/job-list/JobList";
import { useUI } from "../context/UIContext";
import { getPaginatedJobs, startAnalysis, cancelAnalysis } from "@/src/api/analysis";
import type { AnalysisSettingsRequest } from "@/src/lib/types";
import { logger } from "../lib/logger";
import {
    Pagination,
    PaginationContent,
    PaginationItem,
    PaginationLink,
    PaginationNext,
    PaginationPrevious,
} from "@/src/components/ui/pagination";
import { Input } from "@/src/components/ui/input";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/src/components/ui/select";

const fetchJobsPaginated = ([, limit, offset, urlFilter, statusFilter]: [string, number, number, string?, string?]) =>
    getPaginatedJobs(limit, offset, urlFilter, statusFilter).then((res) => {
        return res.unwrapOr({ items: [], total: 0 });
    });

export default function Home() {
    const router = useRouter();
    const { openSettings } = useUI();
    const [isLoading, setIsLoading] = useState(false);
    // selectedResult state removed used for routing
    const [error, setError] = useState<string | null>(null);
    const [currentPage, setCurrentPage] = useState(1);
    const [pageSize, setPageSize] = useState(5);
    const [urlFilter, setUrlFilter] = useState("");
    const [statusFilter, setStatusFilter] = useState("all");

    const {
        data: paginatedData = { items: [], total: 0 },
        mutate,
        isValidating,
    } = useSWR(
        ["jobs", pageSize, (currentPage - 1) * pageSize, urlFilter, statusFilter === "all" ? undefined : statusFilter],
        fetchJobsPaginated,
        {
            refreshInterval: 10_000,
            onError: (e) => setError(e instanceof Error ? e.message : String(e)),
        }
    );

    const { items: jobs, total } = paginatedData;
    const totalPages = Math.ceil(total / pageSize);

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

    const handleViewResult = async (jobId: string) => {
        router.push(`/analysis?id=${jobId}`);
    };
    // Note: using window.location.href or router.push if I import router.
    // Since "use client" is at top, I should import useRouter.

    const handleCancel = async (jobId: string) => {
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
                        <h1 className="text-2xl font-bold">SEO Insikt crawler</h1>
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
                        onClick={openSettings}
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

            {/* Analysis Jobs Section */}
            <div className="space-y-6 mb-6">
                <div className="flex items-center justify-between border-b pb-4">
                    <div>
                        <h2 className="text-xl font-semibold tracking-tight">Recent Analysis</h2>
                        <p className="text-sm text-muted-foreground">
                            Monitor and manage your analysis tasks
                        </p>
                    </div>
                    {total > 0 && (
                        <div className="px-3 py-1 bg-primary/10 text-primary text-xs font-medium rounded-full border border-primary/20">
                            {total} Total
                        </div>
                    )}
                </div>

                <div className="flex flex-col md:flex-row gap-4 p-4 bg-secondary/30 rounded-xl border border-border/50 shadow-sm">
                    <div className="flex-1 relative">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                        <Input
                            placeholder="Search by URL..."
                            value={urlFilter}
                            onChange={(e) => {
                                setUrlFilter(e.target.value);
                                setCurrentPage(1);
                            }}
                            className="pl-10 w-full"
                        />
                    </div>
                    <div className="flex gap-3">
                        <Select
                            value={statusFilter}
                            onValueChange={(val) => {
                                setStatusFilter(val);
                                setCurrentPage(1);
                            }}
                        >
                            <SelectTrigger className="w-full sm:w-40">
                                <SelectValue placeholder="Status" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">All Statuses</SelectItem>
                                <SelectItem value="pending">Pending</SelectItem>
                                <SelectItem value="running">Running</SelectItem>
                                <SelectItem value="completed">Completed</SelectItem>
                                <SelectItem value="failed">Failed</SelectItem>
                                <SelectItem value="cancelled">Cancelled</SelectItem>
                            </SelectContent>
                        </Select>

                        <Select
                            value={pageSize.toString()}
                            onValueChange={(val) => {
                                setPageSize(parseInt(val));
                                setCurrentPage(1);
                            }}
                        >
                            <SelectTrigger className="w-full sm:w-32">
                                <SelectValue placeholder="Show" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="5">5 per page</SelectItem>
                                <SelectItem value="10">10 per page</SelectItem>
                                <SelectItem value="20">20 per page</SelectItem>
                                <SelectItem value="50">50 per page</SelectItem>
                            </SelectContent>
                        </Select>

                        {(urlFilter || statusFilter !== "all") && (
                            <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                    setUrlFilter("");
                                    setStatusFilter("all");
                                    setCurrentPage(1);
                                }}
                                className="text-muted-foreground hover:text-foreground"
                            >
                                Clear
                            </Button>
                        )}
                    </div>
                </div>
            </div>

            <JobList jobs={jobs} onViewResult={handleViewResult} onCancel={handleCancel} />

            {totalPages > 1 && (
                <div className="mt-8">
                    <Pagination>
                        <PaginationContent>
                            <PaginationItem>
                                <PaginationPrevious
                                    href="#"
                                    onClick={(e) => {
                                        e.preventDefault();
                                        if (currentPage > 1) setCurrentPage(currentPage - 1);
                                    }}
                                    className={currentPage === 1 ? "pointer-events-none opacity-50" : "cursor-pointer"}
                                />
                            </PaginationItem>

                            {Array.from({ length: totalPages }).map((_, i) => (
                                <PaginationItem key={i}>
                                    <PaginationLink
                                        href="#"
                                        isActive={currentPage === i + 1}
                                        onClick={(e) => {
                                            e.preventDefault();
                                            setCurrentPage(i + 1);
                                        }}
                                        className="cursor-pointer"
                                    >
                                        {i + 1}
                                    </PaginationLink>
                                </PaginationItem>
                            ))}

                            <PaginationItem>
                                <PaginationNext
                                    href="#"
                                    onClick={(e) => {
                                        e.preventDefault();
                                        if (currentPage < totalPages) setCurrentPage(currentPage + 1);
                                    }}
                                    className={currentPage === totalPages ? "pointer-events-none opacity-50" : "cursor-pointer"}
                                />
                            </PaginationItem>
                        </PaginationContent>
                    </Pagination>
                </div>
            )}
        </main>
    );
}
