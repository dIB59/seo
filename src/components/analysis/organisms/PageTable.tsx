// src/components/analysis/organisms/PageTable.tsx
import { PageAnalysisData } from "@/src/lib/types";
import { Table, TableBody, TableHeader, TableRow, TableHead } from "@/src/components/ui/table";
import { BrokenPageRow, HealthyPageRow } from "../molecules/PageRow";

export function PageTable({ pages, onSelectPage }: { pages: PageAnalysisData[], onSelectPage: (p: number) => void }) {
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
                    <TableHead className="text-center">SEO</TableHead>
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