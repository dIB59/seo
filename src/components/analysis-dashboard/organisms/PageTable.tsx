import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { PageAnalysisData } from "@/src/lib/types";
import { PageRow } from "../molecules/PageRow";
import { GRID_COLS, GRID_GAP } from "../atoms/PageRowAtoms";
import { FileText } from "lucide-react";

export function PageTable({
  pages,
  onSelectPage,
}: {
  pages: PageAnalysisData[];
  onSelectPage: (p: number) => void;
}) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: pages.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 53,
  });

  const virtualItems = virtualizer.getVirtualItems();

  // Empty state
  if (pages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center gap-4 py-20 text-muted-foreground bg-card/40 backdrop-blur border border-border/40 rounded-lg">
        <FileText className="h-12 w-12 opacity-30" />
        <div className="text-center space-y-1">
          <p className="text-sm font-medium">No pages found</p>
          <p className="text-xs">Run an analysis to see page-level results.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-card/40 backdrop-blur border border-border/40 rounded-lg overflow-hidden">
      {/* Header — same grid structure as PageRow for perfect alignment */}
      <div
        className={`grid ${GRID_COLS} ${GRID_GAP} px-4 py-2.5 border-b border-border/30 bg-muted/15 text-[10px] uppercase tracking-[0.08em] font-semibold text-muted-foreground/70`}
      >
        <div className="flex items-center pl-4">Page</div>
        <div className="flex items-center justify-center">Load</div>
        <div className="flex items-center justify-center">Words</div>
        <div className="flex items-center justify-center">H1 / H2 / H3</div>
        <div className="flex items-center justify-center">Images</div>
        <div className="flex items-center justify-center">Int · Ext</div>
        <div className="flex items-center justify-center">Status</div>
        <div className="flex items-center justify-center">SEO</div>
        <div></div>
      </div>

      {/* Scrollable body */}
      <div className="overflow-x-auto">
        <div
          ref={parentRef}
          className="overflow-auto"
          style={{ height: "600px" }}
        >
          <div
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              position: "relative",
            }}
          >
            {virtualItems.map((virtualItem) => {
              const page = pages[virtualItem.index];

              return (
                <div
                  key={virtualItem.key}
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    height: `${virtualItem.size}px`,
                    transform: `translateY(${virtualItem.start}px)`,
                  }}
                >
                  <PageRow
                    page={page}
                    index={virtualItem.index}
                    onClick={onSelectPage}
                  />
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}