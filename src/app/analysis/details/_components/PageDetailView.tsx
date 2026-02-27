"use client";

import { useEffect, useCallback, useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs";
import { Badge } from "@/src/components/ui/badge";
import type { PageDetailData } from "@/src/lib/types";
import PageHeader from "./organisms/PageHeader";
import PageInfoCard from "./molecules/PageInfoCard";
import MetaTab from "./molecules/MetaTab";
import SeoAuditTab from "./molecules/SeoAuditTab";
import HeadingsTab from "./molecules/HeadingsTab";
import ImagesTab from "./molecules/ImagesTab";
import LinksTab from "./molecules/LinksTab";
import ExtractedDataTab from "./molecules/ExtractedDataTab";
import { cn } from "@/src/lib/utils";

interface PageDetailViewProps {
  page: PageDetailData;
  pages: PageDetailData[];
  currentIndex: number;
  onBack: () => void;
  onNavigate: (index: number) => void;
}

export function PageDetailView({
  page,
  pages,
  currentIndex,
  onBack,
  onNavigate,
}: PageDetailViewProps) {
  const [, setSearchOpen] = useState(false);

  const canGoPrev = currentIndex > 0;
  const canGoNext = currentIndex < pages.length - 1;

  const goToPrev = useCallback(() => {
    if (canGoPrev) onNavigate(currentIndex - 1);
  }, [canGoPrev, currentIndex, onNavigate]);

  const goToNext = useCallback(() => {
    if (canGoNext) onNavigate(currentIndex + 1);
  }, [canGoNext, currentIndex, onNavigate]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) return;
      if (e.key === "ArrowLeft") goToPrev();
      if (e.key === "ArrowRight") goToNext();
      if (e.key === "Escape") onBack();
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setSearchOpen(true);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [goToPrev, goToNext, onBack]);

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500 ease-out pb-20">
      {/* Header Section */}
      <div className="sticky top-0 z-20 pb-4 bg-background/80 backdrop-blur-md border-b border-border/40 -mx-6 px-6 pt-4 transition-all duration-300">
        <PageHeader
          page={page}
          pages={pages}
          currentIndex={currentIndex}
          onBack={onBack}
          onNavigate={onNavigate}
        />
      </div>

      {/* Info Card with refined border/shadow */}
      <div className="relative group transition-all duration-300 hover:shadow-sm">
        <div className="absolute -inset-0.5 bg-gradient-to-r from-primary/10 to-primary/0 rounded-xl opacity-0 group-hover:opacity-100 transition duration-500 blur" />
        <div className="relative">
          <PageInfoCard page={page} />
        </div>
      </div>

      <Tabs defaultValue="meta" className="space-y-6">
        <div className="sticky top-[88px] z-10 py-2 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          <TabsList className="w-full h-11 p-1 bg-muted/30 border border-border/40 rounded-lg backdrop-blur text-muted-foreground grid grid-cols-6 gap-1">
            {[
              { value: "meta", label: "Metadata" },
              { value: "seo-audit", label: "SEO Audit" },
              { value: "headings", label: "Headings", count: page.headings.length },
              { value: "images", label: "Images", count: page.image_count },
              { value: "links", label: "Links", count: page.internal_links + page.external_links },
              {
                value: "extracted",
                label: "Extracted",
                count: Object.keys(page.extracted_data || {}).length,
              },
            ].map((tab) => (
              <TabsTrigger
                key={tab.value}
                value={tab.value}
                className="data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm data-[state=active]:border-border/50 text-xs font-medium transition-all duration-200 rounded-md flex items-center justify-center gap-2"
              >
                {tab.label}
                {tab.count !== undefined && tab.count > 0 && (
                  <Badge
                    variant="secondary"
                    className={cn(
                      "px-1.5 py-0 text-[10px] h-4 min-w-4 flex items-center justify-center rounded-sm font-mono transition-colors",
                      "bg-primary/5 text-primary/70 border-primary/10 group-data-[state=active]:bg-primary/10 group-data-[state=active]:text-primary",
                    )}
                  >
                    {tab.count}
                  </Badge>
                )}
              </TabsTrigger>
            ))}
          </TabsList>
        </div>

        <div className="min-h-[400px] animate-in fade-in slide-in-from-bottom-1 duration-300 delay-75">
          <TabsContent
            value="meta"
            className="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <MetaTab page={page} />
          </TabsContent>
          <TabsContent
            value="seo-audit"
            className="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <SeoAuditTab page={page} />
          </TabsContent>
          <TabsContent
            value="headings"
            className="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <HeadingsTab headings={page.headings || []} />
          </TabsContent>
          <TabsContent
            value="images"
            className="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <ImagesTab images={page.images || []} />
          </TabsContent>
          <TabsContent
            value="links"
            className="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <LinksTab links={page.detailed_links || []} />
          </TabsContent>
          <TabsContent
            value="extracted"
            className="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <ExtractedDataTab data={page.extracted_data || {}} />
          </TabsContent>
        </div>
      </Tabs>

      <div className="fixed bottom-0 left-0 right-0 py-2 border-t border-border/40 bg-background/80 backdrop-blur-md flex items-center justify-center text-[11px] text-muted-foreground/70 pointer-events-none z-50">
        <div className="flex items-center gap-4 px-4 py-1.5 rounded-full bg-background/50 border border-border/20 shadow-sm backdrop-blur-sm">
          <div className="flex items-center gap-1.5">
            <kbd className="inline-flex h-5 items-center justify-center rounded border border-border bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground shadow-[0_1px_0_0_rgba(0,0,0,0.05)]">
              ←
            </kbd>
            <kbd className="inline-flex h-5 items-center justify-center rounded border border-border bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground shadow-[0_1px_0_0_rgba(0,0,0,0.05)]">
              →
            </kbd>
            <span>Navigate</span>
          </div>
          <div className="w-px h-3 bg-border/50" />
          <div className="flex items-center gap-1.5">
            <kbd className="inline-flex h-5 items-center justify-center rounded border border-border bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground shadow-[0_1px_0_0_rgba(0,0,0,0.05)]">
              Esc
            </kbd>
            <span>Back</span>
          </div>
        </div>
      </div>
    </div>
  );
}
