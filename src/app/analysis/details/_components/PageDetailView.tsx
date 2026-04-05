"use client";

import { useEffect, useCallback, useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs";
import type { PageAnalysisData } from "@/src/api/analysis";
import PageHeader from "./organisms/PageHeader";
import PageInfoCard from "./molecules/PageInfoCard";
import MetaTab from "./molecules/MetaTab";
import SeoAuditTab from "./molecules/SeoAuditTab";
import HeadingsTab from "./molecules/HeadingsTab";
import ImagesTab from "./molecules/ImagesTab";
import LinksTab from "./molecules/LinksTab";
import ExtractedDataTab from "./molecules/ExtractedDataTab";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { logger } from "@/src/lib";

interface PageDetailViewProps {
    page: PageAnalysisData;
    pages: PageAnalysisData[];
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

    const extractedData = page.extracted_data ?? {};
    const extractedCount = Object.values(extractedData).filter((v) => {
        if (v === null || v === undefined) return false;
        if (typeof v === "string") return v.trim().length > 0;
        if (Array.isArray(v)) return v.length > 0;
        return true;
    }).length;

    const allTabs: { value: string; label: string; count?: number }[] = [
        { value: "meta", label: "Metadata" },
        { value: "seo-audit", label: "SEO Audit" },
        { value: "headings", label: "Headings", count: page.headings?.length || 0 },
        { value: "images", label: "Images", count: page.image_count || 0 },
        {
            value: "links",
            label: "Links",
            count: (page.internal_links || 0) + (page.external_links || 0),
        },
        { value: "extracted", label: "Extracted", count: extractedCount },
    ];

    return (
        <div className="space-y-6 -mx-4 md:-mx-0">
            {/* Sticky Header Section */}
            <div className="sticky top-0 z-30 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 border-b border-border/50 -mx-4 md:-mx-0 px-4 md:px-0">
                <div className="pb-3">
                    <PageHeader
                        page={page}
                        pages={pages}
                        currentIndex={currentIndex}
                        onBack={onBack}
                        onNavigate={onNavigate}
                    />
                </div>
            </div>

            {/* Info Card */}
            <div className="mb-6">
                <PageInfoCard page={page} />
            </div>

            {/* Tabs - contains navigation and content */}
            <Tabs defaultValue="meta" className="space-y-6">
                {/* Tab Navigation - inside Tabs */}
                <div className="pb-3 overflow-x-auto scrollbar-thin -mx-4 md:-mx-0 px-4 md:px-0">
                    <TabsList>
                        {allTabs.map((tab) => (
                            <TabsTrigger key={tab.value} value={tab.value} className="px-3">
                                <span>{tab.label}</span>
                                {tab.count !== undefined && tab.count > 0 && (
                                    <span className="inline-flex items-center justify-center min-w-[1.25rem] h-5 px-1 text-xs font-medium rounded-full bg-primary/15 text-primary">
                                        {tab.count}
                                    </span>
                                )}
                            </TabsTrigger>
                        ))}
                    </TabsList>
                </div>
                <TabsContent value="meta" className="mt-0 focus-visible:outline-none">
                    <MetaTab page={page} />
                </TabsContent>
                <TabsContent value="seo-audit" className="mt-0 focus-visible:outline-none">
                    <SeoAuditTab page={page} />
                </TabsContent>
                <TabsContent value="headings" className="mt-0 focus-visible:outline-none">
                    <HeadingsTab headings={page.headings || []} />
                </TabsContent>
                <TabsContent value="images" className="mt-0 focus-visible:outline-none">
                    <ImagesTab images={page.images || []} />
                </TabsContent>
                <TabsContent value="links" className="mt-0 focus-visible:outline-none">
                    <LinksTab links={page.detailed_links || []} />
                </TabsContent>
                <TabsContent value="extracted" className="mt-0 focus-visible:outline-none">
                    <ExtractedDataTab extractedData={extractedData} />
                </TabsContent>
            </Tabs>

            {/* Keyboard Shortcuts Footer */}
            <div className="fixed bottom-0 left-0 right-0 z-50 border-t border-border/50 bg-background/90 backdrop-blur -mx-4 md:-mx-0 px-4 md:px-0">
                <div className="flex items-center justify-center py-2 max-w-5xl mx-auto">
                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                        <div className="flex items-center gap-1.5">
                            <kbd className="inline-flex h-5 items-center justify-center rounded border border-border/60 bg-muted/60 px-1.5 font-mono text-[10px] font-medium shadow-sm">
                                <ChevronLeft className="h-3 w-3 mr-0.5" />
                                <ChevronRight className="h-3 w-3 ml-0.5" />
                            </kbd>
                            <span className="text-[11px]">Navigate</span>
                        </div>
                        <div className="w-px h-4 bg-border" />
                        <div className="flex items-center gap-1.5">
                            <kbd className="inline-flex h-5 items-center justify-center rounded border border-border/60 bg-muted/60 px-2 font-mono text-[10px] font-medium shadow-sm">
                                Esc
                            </kbd>
                            <span className="text-[11px]">Back</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
