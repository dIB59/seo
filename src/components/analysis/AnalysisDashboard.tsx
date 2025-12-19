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
import { Network } from "lucide-react";
import { OverviewTab } from "./molecules/OverviewTab";

export default function AnalysisDashboard({ data, onBack, onSelectPage }:
    {
        data: CompleteAnalysisResult, onBack: () => void, onSelectPage: (index: number)
            => void
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
                    <TabsTrigger value="graph"><Network className="h-4 w-4" />Graph</TabsTrigger>
                    <TabsTrigger value="overview">Overview</TabsTrigger>
                </TabsList>
                <TabsContent value="issues">
                    <IssuesAccordion issues={data.issues} />
                </TabsContent>
                <TabsContent value="pages">
                    <PageTable pages={data.pages} onSelectPage={onSelectPage} />
                </TabsContent>
                <TabsContent value="graph" className="mt-4">
                    <GraphView
                        data={data}
                        onNodeClick={(page) => {
                            console.log(page);
                            setSelectedPage(data.pages.find(p => p.url === page) ?? null)
                        }}
                    />
                </TabsContent>
                <TabsContent value="overview" className="mt-4">
                    <OverviewTab issues={data.issues} pages={data.pages} />
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