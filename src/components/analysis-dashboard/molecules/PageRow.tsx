// PageRow.tsx - Updated to use grid layout matching header
import { ChevronRight, FileCode, Smartphone, Search } from "lucide-react";
import { cn } from "@/src/lib/utils";
import { PageAnalysisData } from "@/src/lib/types";
import { getLoadTimeColor, getScoreColor } from "@/src/lib/seo-metrics";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  TooltipProvider,
} from "@/src/components/ui/tooltip";

interface RowProps {
  page: PageAnalysisData;
  index: number;
  onClick: (p: number) => void;
}

export function BrokenPageRow({ page, index, onClick }: RowProps) {
  return (
    <div
      onClick={() => onClick(index)}
      className="grid grid-cols-[200px_80px_80px_100px_100px_80px_100px_80px_40px] gap-2 px-4 py-3 border-b cursor-pointer bg-destructive/5 hover:bg-destructive/10 text-destructive items-center"
    >
      <div className="min-w-0">
        <div className="flex flex-col gap-0.5">
          <span className="font-medium text-sm truncate text-foreground">
            {page.url.replace(/^https?:\/\/[^/]+/, "") || "/"}
          </span>
          <span className="text-xs text-muted-foreground truncate">
            {page.title || "No title"}
          </span>
        </div>
      </div>

      <div className="text-center font-medium">
        {page.load_time.toPrecision(2)}s
      </div>

      <div className="text-center">-</div>

      <div className="text-center text-xs">–</div>

      <div className="text-center">
        <div className="flex items-center justify-center gap-1">
          <span>{page.image_count}</span>
          {page.images_without_alt > 0 && (
            <span className="text-destructive/80 text-xs">(-{page.images_without_alt})</span>
          )}
        </div>
      </div>

      <div className="text-center text-xs">
        {page.internal_links}/{page.external_links}
      </div>

      <div className="text-center">-</div>

      <div className="text-center">
        {page.lighthouse_seo ? (
          <span className={cn("text-sm font-medium", getScoreColor(page.lighthouse_seo))}>
            {page.lighthouse_seo.toPrecision(2)}
          </span>
        ) : (
          <span className="text-muted-foreground">-</span>
        )}
      </div>

      <div>
        <ChevronRight className="h-4 w-4 text-muted-foreground" />
      </div>
    </div>
  );
}

export function HealthyPageRow({ page, index, onClick }: RowProps) {
  return (
    <div
      onClick={() => onClick(index)}
      className="grid grid-cols-[200px_80px_80px_100px_100px_80px_100px_80px_40px] gap-2 px-4 py-3 border-b cursor-pointer hover:bg-muted/50 items-center"
    >
      <div className="min-w-0">
        <div className="flex flex-col gap-0.5">
          <span className="font-medium text-sm truncate">
            {page.url.replace(/^https?:\/\/[^/]+/, "") || "/"}
          </span>
          <span className="text-xs text-muted-foreground truncate">
            {page.title || "No title"}
          </span>
        </div>
      </div>

      <div className="text-center">
        <span className={cn("font-medium", getLoadTimeColor(page.load_time))}>
          {page.load_time.toFixed(2)}s
        </span>
      </div>

      <div className="text-center">{page.word_count.toLocaleString()}</div>

      <div className="text-center text-xs">
        {page.h1_count}/{page.h2_count}/{page.h3_count}
      </div>

      <div className="text-center">
        <div className="flex items-center justify-center gap-1">
          <span>{page.image_count}</span>
          {page.images_without_alt > 0 && (
            <span className="text-destructive/80 text-xs">(-{page.images_without_alt})</span>
          )}
        </div>
      </div>

      <div className="text-center text-xs">
        {page.internal_links}/{page.external_links}
      </div>

      <div className="text-center">
        <div className="flex items-center justify-center gap-1.5">
          <Smartphone
            className={cn(
              "h-3.5 w-3.5",
              page.mobile_friendly ? "text-success" : "text-muted-foreground"
            )}
          />
          <FileCode
            className={cn(
              "h-3.5 w-3.5",
              page.has_structured_data ? "text-success" : "text-muted-foreground"
            )}
          />
        </div>
      </div>

      <div className="text-center">
        {page.lighthouse_seo ? (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger className="flex items-center justify-center w-full">
                <span
                  className={cn(
                    "text-sm font-medium",
                    getScoreColor(page.lighthouse_seo)
                  )}
                >
                  {page.lighthouse_seo.toPrecision(3)}
                </span>
              </TooltipTrigger>
              <TooltipContent>
                <p>SEO Score (0-100)</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        ) : (
          <span className="text-muted-foreground">-</span>
        )}
      </div>

      <div>
        <ChevronRight className="h-4 w-4 text-muted-foreground" />
      </div>
    </div>
  );
}