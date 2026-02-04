import { ChevronRight, FileCode, Smartphone } from "lucide-react";
import { TableCell, TableRow } from "../../ui/table";
import { cn } from "@/src/lib/utils";
import { PageAnalysisData } from "@/src/lib/types";
import { getLoadTimeColor, getScoreColor } from "@/src/lib/seo-metrics";

export function BrokenPageRow({ page, onClick }: { page: PageAnalysisData; onClick: () => void }) {
    return (
        <TableRow className="cursor-pointer bg-destructive/5 hover:bg-destructive/10 text-destructive " onClick={onClick}>
            <TableCell className="max-w-[200px]">
                <div className="flex flex-col gap-0.5">
                    <span className="font-medium text-sm truncate text-foreground">
                        {page.url.replace(/^https?:\/\/[^/]+/, "") || "/"}
                    </span>
                    <span className="text-xs text-muted-foreground truncate">
                        {page.title || "No title"}
                    </span>
                </div>
            </TableCell>

            {/* ----- red status code ----- */}
            <TableCell className="text-center">
                <span className="text-destructive font-medium">{page.load_time.toPrecision(2) + "s"}</span>
            </TableCell>

            {/* ----- same data as healthy row ----- */}
            <TableCell className="text-center text-destructive ">-</TableCell>

            <TableCell className="text-center text-xs text-destructive ">
                –
            </TableCell>

            <TableCell className="text-center text-destructive ">
                <div className="flex items-center justify-center gap-1">
                    <span>{page.image_count}</span>
                    {page.images_without_alt > 0 && (
                        <span className="text-destructive/80 text-xs">(-{page.images_without_alt})</span>
                    )}
                </div>
            </TableCell>

            <TableCell className="text-center text-xs">
                {page.internal_links}/{page.external_links}
            </TableCell>

            <TableCell className="text-center">
                <div className="flex items-center justify-center gap-1.5">
                    -
                </div>
            </TableCell>

            <TableCell className="text-center">
                {page.lighthouse_seo ? (
                    <span className={cn("text-sm font-medium", getScoreColor(page.lighthouse_seo))}>
                        {page.lighthouse_seo.toPrecision(2)}
                    </span>
                ) : (
                    <span className="text-muted-foreground">-</span>
                )}
            </TableCell>

            <TableCell>
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
            </TableCell>
        </TableRow>
    );
}



export function HealthyPageRow({ page, onClick }: { page: PageAnalysisData; onClick: () => void }) {
    return (
        <TableRow className="cursor-pointer hover:bg-muted/50" onClick={onClick}>
            <TableCell className="max-w-[200px]">
                <div className="flex flex-col gap-0.5">
                    <span className="font-medium text-sm truncate">{page.url.replace(/^https?:\/\/[^/]+/, "") || "/"}</span>
                    <span className="text-xs text-muted-foreground truncate">{page.title || "No title"}</span>
                </div>
            </TableCell>
            <TableCell className="text-center">
                <span className={cn("font-medium", getLoadTimeColor(page.load_time))}>{page.load_time.toFixed(2)}s</span>
            </TableCell>
            <TableCell className="text-center">{page.word_count.toLocaleString()}</TableCell>
            <TableCell className="text-center">
                <span className="text-xs">
                    {page.h1_count}/{page.h2_count}/{page.h3_count}
                </span>
            </TableCell>
            <TableCell className="text-center">
                <div className="flex items-center justify-center gap-1">
                    <span>{page.image_count}</span>
                    {page.images_without_alt > 0 && (
                        <span className="text-destructive/80 text-xs">(-{page.images_without_alt})</span>
                    )}
                </div>
            </TableCell>
            <TableCell className="text-center">
                <span className="text-xs">
                    {page.internal_links}/{page.external_links}
                </span>
            </TableCell>
            <TableCell className="text-center">
                <div className="flex items-center justify-center gap-1.5">
                    <Smartphone className={cn("h-3.5 w-3.5", page.mobile_friendly ? "text-success" : "text-muted-foreground")} />
                    <FileCode
                        className={cn("h-3.5 w-3.5", page.has_structured_data ? "text-success" : "text-muted-foreground")}
                    />
                </div>
            </TableCell>
            <TableCell className="text-center">
                {page.lighthouse_seo ? (
                    <span className={cn("text-sm font-medium", getScoreColor(page.lighthouse_seo))}>{page.lighthouse_seo.toPrecision(3)}</span>
                ) : (
                    <span className="text-muted-foreground">-</span>
                )}
            </TableCell>
            <TableCell>
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
            </TableCell>
        </TableRow>
    )
}