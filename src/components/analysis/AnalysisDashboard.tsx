import { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types";
import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { PageTable } from "./organisms/PageTable";
import { QuickStatsCard } from "./molecules/QuickStat";
import { ScoreCard } from "./molecules/ScoreCard";
import { SiteHealthCard } from "./molecules/SiteHealthCard";
import { PageDetailModal } from "./organisms/PageDetailModal";

// src/components/analysis/AnalysisDashboard.tsx
export default function AnalysisDashboard({ data }: { data: CompleteAnalysisResult }) {
    const [selectedPage, setSelectedPage] = useState<PageAnalysisData | null>(null);

    return (
        <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <ScoreCard summary={data.summary} issues={data.issues} />
                <SiteHealthCard analysis={data.analysis} pages={data.pages} />
                <QuickStatsCard summary={data.summary} pages={data.pages} />
            </div>

            <Tabs defaultValue="issues">
                <TabsList>
                    <TabsTrigger value="issues">Issues</TabsTrigger>
                    <TabsTrigger value="pages">Pages</TabsTrigger>
                    <TabsTrigger value="visual">Visual</TabsTrigger>
                </TabsList>
                <TabsContent value="issues">
                    <IssuesAccordion issues={data.issues} />
                </TabsContent>
                <TabsContent value="pages">
                    <PageTable pages={data.pages} onSelectPage={setSelectedPage} />
                </TabsContent>
                <TabsContent value="visual" className="mt-4">
                    <SiteVisualizer
                        pages={data.pages}
                        onNodeClick={(page) => setSelectedPage(page)}
                    />
                </TabsContent>
            </Tabs>

            <PageDetailModal
                page={selectedPage}
                open={!!selectedPage}
                onClose={() => setSelectedPage(null)}
            />
        </div>
    );
}