import React from "react";
import { Search } from "lucide-react";
import { Input } from "@/src/components/ui/input";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/src/components/ui/select";
import { Button } from "@/src/components/ui/button";

interface JobFilterBarProps {
    total: number;
    urlFilter: string;
    setUrlFilter: (value: string) => void;
    statusFilter: string;
    setStatusFilter: (value: string) => void;
    pageSize: number;
    setPageSize: (value: number) => void;
    setCurrentPage: (value: number) => void;
}

export const JobFilterBar = function JobFilterBar({
    total,
    urlFilter,
    setUrlFilter,
    statusFilter,
    setStatusFilter,
    pageSize,
    setPageSize,
    setCurrentPage,
}: JobFilterBarProps) {
    const handleUrlChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setUrlFilter(e.target.value);
        setCurrentPage(1);
    };

    const handleStatusChange = (val: string) => {
        setStatusFilter(val);
        setCurrentPage(1);
    };

    const handlePageSizeChange = (val: string) => {
        setPageSize(parseInt(val));
        setCurrentPage(1);
    };

    const handleClear = () => {
        setUrlFilter("");
        setStatusFilter("all");
        setCurrentPage(1);
    };

    return (
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
                        onChange={handleUrlChange}
                        className="pl-10 w-full"
                    />
                </div>
                <div className="flex gap-3">
                    <Select value={statusFilter} onValueChange={handleStatusChange}>
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

                    <Select value={pageSize.toString()} onValueChange={handlePageSizeChange}>
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
                            onClick={handleClear}
                            className="text-muted-foreground hover:text-foreground"
                        >
                            Clear
                        </Button>
                    )}
                </div>
            </div>
        </div>
    );
};
