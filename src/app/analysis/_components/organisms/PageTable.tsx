"use client";

import type { PageAnalysisData } from "@/src/api/analysis";
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
      <div className="overflow-x-auto">
        <div className="overflow-y-scroll" style={{ height: "600px" }}>
          <div
            className={`grid ${GRID_COLS} ${GRID_GAP} px-4 py-2.5 items-center border-b border-border/30 bg-muted/15 text-[10px] uppercase tracking-[0.08em] font-semibold text-muted-foreground/70 sticky top-0 z-10 backdrop-blur bg-card/95`}
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

          <div>
            {pages.map((page, index) => (
              <PageRow key={page.url} page={page} index={index} onClick={onSelectPage} />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
