"use client"

import { Network } from "lucide-react"
import { Card } from "@/src/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs"
import { Badge } from "@/src/components/ui/badge"
import { Table, TableBody, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import type { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types"
import { QuickStatsCard } from "./analysis/molecules/QuickStat"
import { OverviewTab } from "./analysis/molecules/OverviewTab"
import { ScoreCard } from "./analysis/molecules/ScoreCard"
import { SiteHealthCard } from "./analysis/molecules/SiteHealthCard"
import { BrokenPageRow, HealthyPageRow } from "./analysis/molecules/PageRow"
import { IssuesAccordion } from "./analysis/organisms/IssuesAccordion"
import { SiteVisualizer } from "./analysis/organisms/SiteVisualizer"
import { AnalysisHeader } from "./analysis/organisms/AnalysisHeader"



const isBroken = (page: PageAnalysisData) => {
	return page.status_code !== 200;
}



function PagesTab({
	pages,
	onSelectPage,
}: { pages: PageAnalysisData[]; onSelectPage: (index: number) => void }) {
	return (
		<Card>
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead>Page</TableHead>
						<TableHead className="text-center">Load</TableHead>
						<TableHead className="text-center">Words</TableHead>
						<TableHead className="text-center">H1/H2/H3</TableHead>
						<TableHead className="text-center">Images</TableHead>
						<TableHead className="text-center">Links</TableHead>
						<TableHead className="text-center">Status</TableHead>
						<TableHead className="text-center">SEO</TableHead>
						<TableHead className="w-[40px]"></TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{pages.map(

						(page, idx) => (
							isBroken(page) ? (
								<BrokenPageRow key={page.url} page={page} onClick={() => onSelectPage(idx)} />
							) : (
								<HealthyPageRow key={page.url} page={page} onClick={() => onSelectPage(idx)} />
							)
						))}
				</TableBody>
			</Table>
		</Card>
	)
}

interface AnalysisResultsProps {
	result: CompleteAnalysisResult
	onBack: () => void
	onSelectPage?: (index: number) => void
	analysisId: string
}

export function AnalysisResults({ result, onBack, onSelectPage, analysisId }: AnalysisResultsProps) {
	const { analysis, pages, issues, summary } = result

	const handlePageClick = (url: string) => {
		onSelectPage?.(pages.findIndex(p => p.url === url));
	};

	return (
		<div className="space-y-6">

			<AnalysisHeader
				onBack={onBack}
				result={result}
			/>


			<div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
				<ScoreCard summary={summary} issues={issues} />
				<SiteHealthCard analysis={analysis} pages={pages} />
				<QuickStatsCard summary={summary} pages={pages} />
			</div>

			<Tabs defaultValue="issues" className="space-y-4">
				<TabsList>
					<TabsTrigger value="issues" className="gap-2">
						Issues
						<Badge variant="secondary" className="h-5 px-1.5 text-xs">
							{issues.length}
						</Badge>
					</TabsTrigger>
					<TabsTrigger value="pages" className="gap-2">
						Pages
						<Badge variant="secondary" className="h-5 px-1.5 text-xs">
							{pages.length}
						</Badge>
					</TabsTrigger>
					<TabsTrigger value="graph" className="gap-2"><Network className="h-4 w-4" /> Graph</TabsTrigger>
					<TabsTrigger value="overview">Overview</TabsTrigger>
				</TabsList>

				<TabsContent value="issues" className="mt-4">
					<IssuesAccordion issues={issues} />
				</TabsContent>

				<TabsContent value="pages">
					<PagesTab
						pages={pages}
						onSelectPage={(index) => {
							console.log(index)
							if (onSelectPage) onSelectPage(index)
						}}
					/>
				</TabsContent>

				<TabsContent value="graph" className="h-[700px]">
					<SiteVisualizer data={result} onNodeClick={(page) => handlePageClick(page.url)} />
				</TabsContent>

				<TabsContent value="overview" className="mt-4">
					<OverviewTab issues={issues} pages={pages} />
				</TabsContent>
			</Tabs>

		</div >
	)
}
