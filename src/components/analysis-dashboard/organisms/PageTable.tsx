// PageTableVirtual.tsx
import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { PageAnalysisData } from "@/src/lib/types";
import { BrokenPageRow, HealthyPageRow } from "../molecules/PageRow";

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

  return (
    <div className="border rounded-md">
      {/* Header - use same structure as rows */}
      <div className="grid grid-cols-[200px_80px_80px_100px_100px_80px_100px_80px_40px] gap-2 px-4 py-3 border-b font-medium text-sm bg-muted/50">
        <div>Page</div>
        <div className="text-center">Load</div>
        <div className="text-center">Words</div>
        <div className="text-center">H1/H2/H3</div>
        <div className="text-center">Images</div>
        <div className="text-center">Links</div>
        <div className="text-center">Status</div>
        <div className="text-center">SEO</div>
        <div></div>
      </div>

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
            const isBroken = page.status_code && (page.status_code >= 400 || page.status_code < 200);
            const RowComponent = isBroken ? BrokenPageRow : HealthyPageRow;

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
                <RowComponent
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
  );
}