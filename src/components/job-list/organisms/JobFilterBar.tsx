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
        <div className="flex flex-col sm:flex-row gap-4 items-center justify-between p-1">
            <div className="relative w-full sm:w-auto sm:flex-1 max-w-md">
                <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/60" />
                <Input
                    placeholder="Filter by URL..."
                    value={urlFilter}
                    onChange={handleUrlChange}
                    className="pl-9 w-full bg-background/50 border-input/60 focus:bg-background transition-colors h-9"
                />
            </div>

            <div className="flex items-center gap-2 w-full sm:w-auto">
                <Select value={statusFilter} onValueChange={handleStatusChange}>
                    <SelectTrigger className="w-[130px] h-9 bg-background/50 border-input/60">
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
                    <SelectTrigger className="w-[80px] h-9 bg-background/50 border-input/60">
                        <span className="text-muted-foreground mr-1">Show</span>
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="5">5</SelectItem>
                        <SelectItem value="10">10</SelectItem>
                        <SelectItem value="20">20</SelectItem>
                        <SelectItem value="50">50</SelectItem>
                    </SelectContent>
                </Select>

                {(urlFilter || statusFilter !== "all") && (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={handleClear}
                        className="h-9 px-3 text-muted-foreground hover:text-foreground"
                    >
                        Reset
                    </Button>
                )}

                {total > 0 && (
                    <div className="ml-auto sm:ml-2 h-9 px-3 flex items-center justify-center bg-muted/50 text-muted-foreground text-xs font-medium rounded-md border border-border/50">
                        <span className="opacity-70 mr-1">Total:</span> {total}
                    </div>
                )}
            </div>
        </div>
    );
};
