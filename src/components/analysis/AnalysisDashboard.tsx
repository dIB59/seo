import { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types";
import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { PageTable } from "./organisms/PageTable";
import { QuickStatsCard } from "./molecules/QuickStat";
import { ScoreCard } from "./molecules/ScoreCard";
import { SiteHealthCard } from "./molecules/SiteHealthCard";
import { PageDetailModal } from "./organisms/PageDetailModal";
import { IssuesAccordion } from "./organisms/IssuesAccordion";
import { GraphView } from "../graph-view";
import { AnalysisHeader } from "./organisms/AnalysisHeader";

export default function AnalysisDashboard({ data, onBack, onSelectPage, analysisId }:
    {
        data: CompleteAnalysisResult, onBack: () => void, onSelectPage: (index: number)
            => void, analysisId: string
    }) {
    const [selectedPage, setSelectedPage] = useState<PageAnalysisData | null>(null);

    return (
        <div className="space-y-6">
            <AnalysisHeader
                onBack={onBack}
                result={data}
            />
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <ScoreCard summary={data.summary} issues={data.issues} />
                <SiteHealthCard analysis={data.analysis} pages={data.pages} />
                <QuickStatsCard summary={data.summary} pages={data.pages} />
            </div>

            <Tabs defaultValue="issues">
                <TabsList>
                    <TabsTrigger value="issues">Issues</TabsTrigger>
                    <TabsTrigger value="pages">Pages</TabsTrigger>
                    <TabsTrigger value="pages">Raw Data</TabsTrigger>
                </TabsList>
                <TabsContent value="issues">
                    <IssuesAccordion issues={data.issues} />
                </TabsContent>
                <TabsContent value="pages">
                    <PageTable pages={data.pages} onSelectPage={onSelectPage} />
                </TabsContent>
                <TabsContent value="visual" className="mt-4">
                    <GraphView
                        data={data}
                        onNodeClick={(page) => setSelectedPage(data.pages.find(p => p.url === page) ?? null)}
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