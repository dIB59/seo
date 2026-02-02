// src/components/analysis/organisms/PageTable.tsx
import { PageAnalysisData } from "@/src/lib/types";
import { Table, TableBody, TableHeader, TableRow, TableHead } from "@/src/components/ui/table";
import { BrokenPageRow, HealthyPageRow } from "../molecules/PageRow";
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from "@/src/components/ui/tooltip";
import { Gauge } from "lucide-react";

export function PageTable({ pages, onSelectPage }: { pages: PageAnalysisData[], onSelectPage: (p: number) => void }) {
    // Check if any page has Lighthouse data
    const hasLighthouseData = pages.some((p) => p.lighthouse_seo !== null);

    return (
        <Table>
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
                        {hasLighthouseData ? (
                            <TooltipProvider>
                                <Tooltip>
                                    <TooltipTrigger className="flex items-center justify-center gap-1">
                                        <Gauge className="h-3.5 w-3.5" />
                                        <span>SEO</span>
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>Lighthouse SEO Score</p>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        ) : (
                            "SEO"
                        )}
                    </TableHead>
                    <TableHead className="w-[40px]"></TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {pages.map((page, idx) => (
                    page.status_code && (page.status_code >= 400 || page.status_code < 200)
                        ? <BrokenPageRow key={idx} page={page} onClick={() => onSelectPage(idx)} />
                        : <HealthyPageRow key={idx} page={page} onClick={() => onSelectPage(idx)} />
                ))}
            </TableBody>
        </Table>
    );
}