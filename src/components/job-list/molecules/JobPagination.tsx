import {
    Pagination,
    PaginationContent,
    PaginationItem,
    PaginationLink,
    PaginationNext,
    PaginationPrevious,
    PaginationEllipsis,
} from "@/src/components/ui/pagination";
import { Input } from "@/src/components/ui/input";
import React from "react";

interface JobPaginationProps {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
    className?: string;
}

const generatePaginationItems = (currentPage: number, totalPages: number) => {
    if (totalPages <= 7) {
        return Array.from({ length: totalPages }, (_, i) => i + 1);
    }

    const items: (number | "ellipsis")[] = [];

    if (currentPage <= 4) {
        // Near the start: 1 2 3 4 5 ... totalPages
        for (let i = 1; i <= 5; i++) items.push(i);
        items.push("ellipsis");
        items.push(totalPages);
    } else if (currentPage >= totalPages - 3) {
        // Near the end: 1 ... totalPages-4 totalPages-3 totalPages-2 totalPages-1 totalPages
        items.push(1);
        items.push("ellipsis");
        for (let i = totalPages - 4; i <= totalPages; i++) items.push(i);
    } else {
        // Middle: 1 ... cp-1 cp cp+1 ... totalPages
        items.push(1);
        items.push("ellipsis");
        items.push(currentPage - 1);
        items.push(currentPage);
        items.push(currentPage + 1);
        items.push("ellipsis");
        items.push(totalPages);
    }

    return items;
};

export const JobPagination = function JobPagination({ currentPage, totalPages, onPageChange, className }: JobPaginationProps) {
    const [editingIndex, setEditingIndex] = React.useState<number | null>(null);
    const [inputValue, setInputValue] = React.useState("");

    if (totalPages <= 1) return null;

    const handleJump = () => {
        const page = parseInt(inputValue, 10);
        if (!isNaN(page) && page >= 1 && page <= totalPages) {
            onPageChange(page);
        }
        setEditingIndex(null);
        setInputValue("");
    };

    return (
        <div className={`mt-8 ${className || ""}`}>
            <Pagination>
                <PaginationContent>
                    <PaginationItem>
                        <PaginationPrevious
                            href="#"
                            onClick={(e: React.MouseEvent) => {
                                e.preventDefault();
                                if (currentPage > 1) onPageChange(currentPage - 1);
                            }}
                            className={currentPage === 1 ? "pointer-events-none opacity-50" : "cursor-pointer"}
                        />
                    </PaginationItem>

                    {generatePaginationItems(currentPage, totalPages).map((item, i) => (
                        <PaginationItem key={i}>
                            {item === "ellipsis" ? (
                                editingIndex === i ? (
                                    <Input
                                        autoFocus
                                        className="h-9 w-12 px-1 text-center"
                                        value={inputValue}
                                        onChange={(e) => setInputValue(e.target.value)}
                                        onBlur={handleJump}
                                        onKeyDown={(e) => {
                                            if (e.key === "Enter") handleJump();
                                            if (e.key === "Escape") {
                                                setEditingIndex(null);
                                                setInputValue("");
                                            }
                                        }}
                                    />
                                ) : (
                                    <PaginationEllipsis
                                        className="cursor-pointer hover:bg-accent hover:text-accent-foreground rounded-md transition-colors"
                                        onClick={() => setEditingIndex(i)}
                                    />
                                )
                            ) : (
                                <PaginationLink
                                    href="#"
                                    isActive={currentPage === item}
                                    onClick={(e: React.MouseEvent) => {
                                        e.preventDefault();
                                        onPageChange(item as number);
                                    }}
                                    className="cursor-pointer"
                                >
                                    {item}
                                </PaginationLink>
                            )}
                        </PaginationItem>
                    ))}

                    <PaginationItem>
                        <PaginationNext
                            href="#"
                            onClick={(e: React.MouseEvent) => {
                                e.preventDefault();
                                if (currentPage < totalPages) onPageChange(currentPage + 1);
                            }}
                            className={currentPage === totalPages ? "pointer-events-none opacity-50" : "cursor-pointer"}
                        />
                    </PaginationItem>
                </PaginationContent>
            </Pagination>
        </div>
    );
};


