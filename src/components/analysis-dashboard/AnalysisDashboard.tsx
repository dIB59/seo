import { useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { PageTable } from "./organisms/PageTable";
import { QuickStatsCard } from "./molecules/QuickStat";
import { ScoreCard } from "./molecules/ScoreCard";
import { SiteHealthCard } from "./molecules/SiteHealthCard";
import { PageDetailModal } from "./organisms/PageDetailModal";
import { IssuesAccordion } from "./organisms/IssuesAccordion";
import { AnalysisHeader } from "./organisms/AnalysisHeader";
import { Network, Activity, AlertTriangle, FileText } from "lucide-react";
import { OverviewTab } from "./molecules/OverviewTab";
import GraphView from "../graph-view/GraphView";
import { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types";

export default function AnalysisDashboard({
  data,
  onBack,
  onSelectPage,
}: {
  data: CompleteAnalysisResult;
  onBack: () => void;
  onSelectPage: (index: number) => void;
}) {
  const [selectedPage, setSelectedPage] = useState<PageAnalysisData | null>(null);

  return (
    <div className="min-h-screen bg-background text-foreground relative overflow-hidden">
      {/* Ambient Background */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-primary/5 via-background to-background pointer-events-none" />

      <div className="relative z-10 p-4 space-y-6 max-w-[1600px] mx-auto">
        <AnalysisHeader onBack={onBack} result={data} />

        {/* Key Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 animate-in fade-in slide-in-from-bottom-4 duration-500 delay-100">
          <ScoreCard summary={data.summary} pages={data.pages} issues={data.issues} />
          <SiteHealthCard analysis={data.analysis} pages={data.pages} />
          <QuickStatsCard summary={data.summary} pages={data.pages} />
        </div>

        {/* Data View Tabs */}
        <Tabs
          defaultValue="issues"
          className="animate-in fade-in slide-in-from-bottom-4 duration-500 delay-200"
        >
          <div className="flex items-center justify-between mb-4">
            <TabsList className="bg-muted/30 border border-border/40 p-1 h-auto backdrop-blur-sm shadow-sm">
              <TabsTrigger
                value="issues"
                className="gap-2 px-4 py-2 data-[state=active]:bg-background data-[state=active]:shadow-sm transition-all duration-300 hover:bg-muted/20 hover:text-foreground"
              >
                <AlertTriangle className="h-4 w-4" />
                Issues
              </TabsTrigger>
              <TabsTrigger
                value="pages"
                className="gap-2 px-4 py-2 data-[state=active]:bg-background data-[state=active]:shadow-sm transition-all duration-300 hover:bg-muted/20 hover:text-foreground"
              >
                <FileText className="h-4 w-4" />
                Pages
              </TabsTrigger>
              <TabsTrigger
                value="graph"
                className="gap-2 px-4 py-2 data-[state=active]:bg-background data-[state=active]:shadow-sm transition-all duration-300 hover:bg-muted/20 hover:text-foreground"
              >
                <Network className="h-4 w-4" />
                Site Graph
              </TabsTrigger>
              <TabsTrigger
                value="overview"
                className="gap-2 px-4 py-2 data-[state=active]:bg-background data-[state=active]:shadow-sm transition-all duration-300 hover:bg-muted/20 hover:text-foreground"
              >
                <Activity className="h-4 w-4" />
                Overview
              </TabsTrigger>
            </TabsList>
          </div>

          <div className="mt-4 min-h-[500px]">
            <TabsContent value="issues" className="mt-0">
              <IssuesAccordion issues={data.issues} />
            </TabsContent>
            <TabsContent value="pages" className="mt-0">
              <PageTable pages={data.pages} onSelectPage={onSelectPage} />
            </TabsContent>
            <TabsContent
              value="graph"
              className="mt-0 h-[600px] rounded-lg overflow-hidden border border-border/40 bg-background/50"
            >
              <GraphView
                data={data}
                onNodeClick={(page) => {
                  console.log(page);
                  setSelectedPage(data.pages.find((p) => p.url === page) ?? null);
                }}
              />
            </TabsContent>
            <TabsContent value="overview" className="mt-0">
              <OverviewTab issues={data.issues} pages={data.pages} />
            </TabsContent>
          </div>
        </Tabs>
      </div>

      <PageDetailModal
        page={selectedPage}
        open={!!selectedPage}
        onClose={() => setSelectedPage(null)}
      />
    </div>
  );
}
