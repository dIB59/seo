// src/components/analysis/organisms/PageTable.tsx
import { PageAnalysisData } from "@/src/lib/types";
import { Table, TableBody, TableHeader, TableRow, TableHead } from "@/src/components/ui/table";
import { BrokenPageRow, HealthyPageRow } from "../molecules/PageRow";
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from "@/src/components/ui/tooltip";
import { Search } from "lucide-react";
import { useVirtualizer } from '@tanstack/react-virtual';
import React from "react";

export function PageTable({ pages, onSelectPage }: { pages: PageAnalysisData[], onSelectPage: (p: number) => void }) {
    
    const parentRef = React.useRef(null)

    // The virtualizer
    const rowVirtualizer = useVirtualizer({
        count: pages.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 35,
        overscan: 10
    })
    
    return (
        <Table ref={parentRef} className="overflow-auto" style={{ maxHeight: "calc(100vh - 200px)" }}>
            <TableHeader>
                <TableRow>
                    <TableHead>Page</TableHead>
                    <TableHead className="text-center">Load</TableHead>
                    <TableHead className="text-center">Words</TableHead>
                    <TableHead className="text-center">H1/H2/H3</TableHead>
                    <TableHead className="text-center">Images</TableHead>
                    <TableHead className="text-center">Links</TableHead>
                    <TableHead className="text-center">Status</TableHead>
                    <TableHead className="text-center">
                        <TooltipProvider>
                            <Tooltip>
                                <TooltipTrigger className="flex items-center justify-center gap-1">
                                    <Search className="h-3.5 w-3.5" />
                                    <span>SEO</span>
                                </TooltipTrigger>
                                <TooltipContent>
                                    <p>SEO Score (0-100)</p>
                                </TooltipContent>
                            </Tooltip>
                        </TooltipProvider>
                    </TableHead>
                    <TableHead className="w-[40px]"></TableHead>
                </TableRow>
            </TableHeader>
            <TableBody 
            style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
            }}>
                {rowVirtualizer.getVirtualItems().map((virtualItem)  => {
                    const page = pages[virtualItem.index];
                    return (page.status_code && (page.status_code >= 400 || page.status_code < 200)
                        ? <BrokenPageRow key={virtualItem.key} page={page} onClick={() => onSelectPage(virtualItem.index)} />
                        : <HealthyPageRow key={virtualItem.key} page={page} onClick={() => onSelectPage(virtualItem.index)} />
                    );
                })}
            </TableBody>
        </Table>
    );
}