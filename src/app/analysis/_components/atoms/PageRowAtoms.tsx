import { cn } from "@/src/lib/utils";
import { getLoadTimeColor } from "@/src/lib/seo-metrics";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  TooltipProvider,
} from "@/src/components/ui/tooltip";
import { Smartphone, FileCode, ChevronRight, AlertTriangle } from "lucide-react";

// Column width definitions - shared between header and rows
export const GRID_COLS =
  "grid-cols-[minmax(180px,1.5fr)_80px_80px_100px_100px_90px_100px_80px_36px]";
export const GRID_GAP = "gap-1.5";

// Common cell wrapper classes
export const CELL = {
  base: "flex items-center justify-center text-[13px]",
  left: "flex items-center",
  truncate: "min-w-0 truncate",
};

// Status-specific styles
export const STYLES = {
  healthy: {
    row: "hover:bg-primary/[0.03] group",
    loadTime: (loadTime: number) =>
      cn("font-mono text-[13px] font-medium tabular-nums", getLoadTimeColor(loadTime)),
    text: "text-foreground",
    subtext: "text-muted-foreground",
  },
  broken: {
    row: "bg-destructive/[0.04] hover:bg-destructive/[0.08] group",
    loadTime: "text-destructive font-mono text-[13px] font-medium tabular-nums",
    text: "text-destructive",
    subtext: "text-destructive/60",
  },
};

// Status icons with tooltips
export function StatusIcons({
  mobileFriendly,
  hasStructuredData,
}: {
  mobileFriendly: boolean;
  hasStructuredData: boolean;
}) {
  return (
    <TooltipProvider delayDuration={200}>
      <div className="flex items-center justify-center gap-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <div
              className={cn(
                "p-1 rounded-md transition-colors",
                mobileFriendly
                  ? "text-success bg-success/10"
                  : "text-muted-foreground/50 bg-muted/30",
              )}
            >
              <Smartphone className="h-3.5 w-3.5" />
            </div>
          </TooltipTrigger>
          <TooltipContent side="top" className="text-xs">
            {mobileFriendly ? "Mobile friendly" : "Not mobile friendly"}
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <div
              className={cn(
                "p-1 rounded-md transition-colors",
                hasStructuredData
                  ? "text-success bg-success/10"
                  : "text-muted-foreground/50 bg-muted/30",
              )}
            >
              <FileCode className="h-3.5 w-3.5" />
            </div>
          </TooltipTrigger>
          <TooltipContent side="top" className="text-xs">
            {hasStructuredData ? "Has structured data" : "No structured data"}
          </TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  );
}

// SEO Score cell — compact colored badge
export function SeoScore({ score }: { score: number | null }) {
  if (!score) return <span className="text-muted-foreground/40 text-xs">—</span>;

  return (
    <TooltipProvider delayDuration={200}>
      <Tooltip>
        <TooltipTrigger className="flex items-center justify-center w-full">
          <span
            className={cn(
              "inline-flex items-center justify-center min-w-[36px] px-1.5 py-0.5 rounded-md text-xs font-semibold font-mono tabular-nums",
              score >= 80
                ? "bg-success/15 text-success"
                : score >= 50
                  ? "bg-warning/15 text-warning"
                  : "bg-destructive/15 text-destructive",
            )}
          >
            {score.toPrecision(3)}
          </span>
        </TooltipTrigger>
        <TooltipContent side="top" className="text-xs">
          SEO Score (0–100)
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

// Helper to format URL path
function formatUrlPath(url: string): string {
  return url.replace(/^https?:\/\/[^/]+/, "") || "/";
}

// Page info cell (first column) — path + title with status indicator
export function PageInfo({
  url,
  title,
  isBroken,
  statusCode,
}: {
  url: string;
  title: string | null;
  isBroken: boolean;
  statusCode?: number | null;
}) {
  const styles = isBroken ? STYLES.broken : STYLES.healthy;

  return (
    <div className={cn(CELL.left, "min-w-0 gap-2.5")}>
      {/* Status dot */}
      <div
        className={cn(
          "w-1.5 h-1.5 rounded-full shrink-0",
          isBroken ? "bg-destructive" : "bg-success",
        )}
      />
      <div className="flex flex-col gap-0 w-full min-w-0">
        <div className="flex items-center gap-1.5">
          <span className={cn("font-medium text-[13px] truncate leading-tight", styles.text)}>
            {formatUrlPath(url)}
          </span>
          {isBroken && statusCode && (
            <span className="shrink-0 text-[10px] font-mono font-semibold px-1 py-0 rounded bg-destructive/15 text-destructive">
              {statusCode}
            </span>
          )}
        </div>
        <span className={cn("text-[11px] truncate leading-tight", styles.subtext)}>
          {title || "No title"}
        </span>
      </div>
    </div>
  );
}

// Load time cell — monospace + contextual icon
export function LoadTime({ loadTime, isBroken }: { loadTime: number; isBroken: boolean }) {
  if (isBroken) {
    return <span className={STYLES.broken.loadTime}>{loadTime.toPrecision(2)}s</span>;
  }

  return <span className={STYLES.healthy.loadTime(loadTime)}>{loadTime.toFixed(2)}s</span>;
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
  return (
    <div className="flex items-center justify-center gap-1 text-[13px]">
      <span className="font-mono tabular-nums">{count}</span>
      {withoutAlt > 0 && !isBroken && (
        <TooltipProvider delayDuration={200}>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="flex items-center gap-0.5 text-[10px] font-medium text-warning bg-warning/10 px-1 py-0 rounded">
                <AlertTriangle className="h-2.5 w-2.5" />
                {withoutAlt}
              </span>
            </TooltipTrigger>
            <TooltipContent side="top" className="text-xs">
              {withoutAlt} image{withoutAlt !== 1 ? "s" : ""} missing alt text
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}
    </div>
  );
}

// Words count cell — formatted with locale
export function WordsCell({ count, isBroken }: { count: number; isBroken: boolean }) {
  return (
    <div className={cn(CELL.base, "font-mono tabular-nums")}>
      {isBroken ? <span className="text-muted-foreground/40">—</span> : count.toLocaleString()}
    </div>
  );
}

// Heading counts cell (H1/H2/H3) — individual labels
export function HeadingCounts({
  h1,
  h2,
  h3,
  isBroken,
}: {
  h1: number;
  h2: number;
  h3: number;
  isBroken: boolean;
}) {
  if (isBroken) {
    return <div className={cn(CELL.base, "text-muted-foreground/40")}>—</div>;
  }

  return (
    <div className={cn(CELL.base, "gap-1")}>
      <span
        className={cn(
          "text-[11px] font-mono tabular-nums px-1 py-0 rounded",
          h1 === 0
            ? "text-destructive/70 bg-destructive/8"
            : h1 > 1
              ? "text-warning/80 bg-warning/8"
              : "text-foreground/70 bg-muted/40",
        )}
      >
        {h1}
      </span>
      <span className="text-border/80">/</span>
      <span className="text-[11px] font-mono tabular-nums text-foreground/70">{h2}</span>
      <span className="text-border/80">/</span>
      <span className="text-[11px] font-mono tabular-nums text-foreground/70">{h3}</span>
    </div>
  );
}

// Links count cell — internal/external with labels
export function LinksCell({
  internal,
  external,
  isBroken,
}: {
  internal: number;
  external: number;
  isBroken: boolean;
}) {
  if (isBroken) {
    return <div className={cn(CELL.base, "text-muted-foreground/40")}>—</div>;
  }

  return (
    <div className={cn(CELL.base, "gap-1.5 text-[12px] font-mono tabular-nums")}>
      <span className="text-foreground/80">{internal}</span>
      <span className="text-border/60">·</span>
      <span className="text-muted-foreground/70">{external}</span>
    </div>
  );
}

// Chevron/action cell — reveals on hover
export function ChevronCell() {
  return (
    <div
      className={cn(CELL.base, "opacity-0 group-hover:opacity-100 transition-opacity duration-200")}
    >
      <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
    </div>
  );
}
