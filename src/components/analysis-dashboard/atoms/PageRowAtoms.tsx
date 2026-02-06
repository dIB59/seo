import { cn } from "@/src/lib/utils";
import { getLoadTimeColor, getScoreColor } from "@/src/lib/seo-metrics";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  TooltipProvider,
} from "@/src/components/ui/tooltip";
import { Smartphone, FileCode, ChevronRight } from "lucide-react";

// Column width definitions - shared between header and rows
export const GRID_COLS = "grid-cols-[200px_80px_80px_100px_100px_80px_100px_80px_40px]";
export const GRID_GAP = "gap-2";

// Common cell wrapper classes
export const CELL = {
  base: "flex items-center justify-center",
  left: "flex items-center",
  truncate: "min-w-0 truncate",
};

// Status-specific styles
export const STYLES = {
  healthy: {
    row: "hover:bg-muted/50",
    loadTime: (loadTime: number) => cn("font-medium", getLoadTimeColor(loadTime)),
    text: "text-foreground",
    subtext: "text-muted-foreground",
  },
  broken: {
    row: "bg-destructive/5 hover:bg-destructive/10 text-destructive",
    loadTime: "text-destructive font-medium",
    text: "text-foreground",
    subtext: "text-destructive/80",
  },
};

// Shared icon component
export function StatusIcons({
  mobileFriendly,
  hasStructuredData,
}: {
  mobileFriendly: boolean;
  hasStructuredData: boolean;
}) {
  return (
    <div className="flex items-center justify-center gap-1.5">
      <Smartphone
        className={cn(
          "h-3.5 w-3.5",
          mobileFriendly ? "text-success" : "text-muted-foreground"
        )}
      />
      <FileCode
        className={cn(
          "h-3.5 w-3.5",
          hasStructuredData ? "text-success" : "text-muted-foreground"
        )}
      />
    </div>
  );
}

// SEO Score cell with tooltip
export function SeoScore({ score }: { score: number | null }) {
  if (!score) return <span className="text-muted-foreground">-</span>;

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger className="flex items-center justify-center w-full">
          <span className={cn("text-sm font-medium", getScoreColor(score))}>
            {score.toPrecision(3)}
          </span>
        </TooltipTrigger>
        <TooltipContent>
          <p>SEO Score (0-100)</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

// Helper to format URL path
export function formatUrlPath(url: string): string {
  return url.replace(/^https?:\/\/[^/]+/, "") || "/";
}

// Page info cell (first column)
export function PageInfo({
  url,
  title,
  isBroken,
}: {
  url: string;
  title: string | null;
  isBroken: boolean;
}) {
  const styles = isBroken ? STYLES.broken : STYLES.healthy;

  return (
    <div className={cn(CELL.left, "min-w-0")}>
      <div className="flex flex-col gap-0.5 w-full">
        <span className={cn("font-medium text-sm truncate", styles.text)}>
          {formatUrlPath(url)}
        </span>
        <span className={cn("text-xs truncate", styles.subtext)}>
          {title || "No title"}
        </span>
      </div>
    </div>
  );
}

// Load time cell
export function LoadTime({
  loadTime,
  isBroken,
}: {
  loadTime: number;
  isBroken: boolean;
}) {
  if (isBroken) {
    return <span className={STYLES.broken.loadTime}>{loadTime.toPrecision(2)}s</span>;
  }

  return (
    <span className={STYLES.healthy.loadTime(loadTime)}>{loadTime.toFixed(2)}s</span>
  );
}

// Images cell with alt text warning
export function ImageCount({
  count,
  withoutAlt,
  isBroken,
}: {
  count: number;
  withoutAlt: number;
  isBroken: boolean;
}) {
  const warningClass = isBroken ? "text-destructive/80" : "text-destructive/80";

  return (
    <div className="flex items-center justify-center gap-1">
      <span>{count}</span>
      {withoutAlt > 0 && (
        <span className={cn("text-xs", warningClass)}>(-{withoutAlt})</span>
      )}
    </div>
  );
}

// Words count cell
export function WordsCell({ count, isBroken }: { count: number; isBroken: boolean }) {
  return <div className={CELL.base}>{isBroken ? "-" : count.toLocaleString()}</div>;
}

// Heading counts cell (H1/H2/H3)
export function HeadingCounts({ h1, h2, h3, isBroken }: { h1: number; h2: number; h3: number; isBroken: boolean }) {
  return <div className={cn(CELL.base, "text-xs")}>
    {isBroken ? "–" : `${h1}/${h2}/${h3}`}
  </div>;
}

// Links count cell
export function LinksCell({ internal, external, isBroken }: { internal: number; external: number; isBroken: boolean }) {
  return <div className={cn(CELL.base, "text-xs")}>
    {isBroken ? "0/0" : `${internal}/${external}`}
  </div>;
}

// Chevron/action cell
export function ChevronCell() {
  return (
    <div className={CELL.base}>
      <ChevronRight className="h-4 w-4 text-muted-foreground" />
    </div>
  );
}