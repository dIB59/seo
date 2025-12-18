"use client"

import {
	ArrowLeft,
	FileText,
	Download,
	ExternalLink,
	ChevronDown,
	Table as TableIcon,
	Network,
} from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Card } from "@/src/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs"
import { Badge } from "@/src/components/ui/badge"
import { Table, TableBody, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/src/components/ui/dropdown-menu"
import type { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types"
import { generatePDF, downloadTextReport, downloadCSVReport } from "@/src/lib/export-utils"
import { QuickStatsCard } from "./analysis/molecules/QuickStat"
import { OverviewTab } from "./analysis/molecules/OverviewTab"
import { ScoreCard } from "./analysis/molecules/ScoreCard"
import { SiteHealthCard } from "./analysis/molecules/SiteHealthCard"
import { BrokenPageRow, HealthyPageRow } from "./analysis/molecules/PageRow"
import { IssuesAccordion } from "./analysis/organisms/IssuesAccordion"
import { SiteVisualizer } from "./analysis/organisms/SiteVisualizer"



const isBroken = (page: PageAnalysisData) => {
	return page.status_code !== 200;
}

// ============================================================================
// PAGE TABLE COMPONENTS
// ============================================================================
function PageDetailRow({ page, onClick }: { page: PageAnalysisData; onClick: () => void }) {
	return isBroken(page) ? (
		<BrokenPageRow page={page} onClick={onClick} />
	) : (
		<HealthyPageRow page={page} onClick={onClick} />
	);
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

	const handleDownloadPDF = async () => {
		await generatePDF(result)
	}

	const handleDownloadText = async () => {
		await downloadTextReport(result)
	}

	const handleDownloadCSV = async () => {
		await downloadCSVReport(result)
	}


	return (
		<div className="space-y-6">
			{/* Header */}
			<div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
				<div className="flex items-center gap-3 min-w-0">
					<Button variant="ghost" size="icon" onClick={onBack} className="shrink-0">
						<ArrowLeft className="h-4 w-4" />
					</Button>
					<div className="min-w-0">
						<div className="flex items-center gap-2">
							<h2 className="text-xl font-semibold truncate">{analysis.url}</h2>
							<a href={analysis.url} target="_blank" rel="noopener noreferrer" className="shrink-0">
								<ExternalLink className="h-4 w-4 text-muted-foreground hover:text-foreground" />
							</a>
						</div>
						<p className="text-sm text-muted-foreground">
							{pages.length} pages analyzed · {new Date(analysis.completed_at || "").toLocaleDateString()}
						</p>
					</div>
				</div>
				<DropdownMenu>
					<DropdownMenuTrigger asChild>
						<Button variant="outline" className="shrink-0 bg-transparent">
							<Download className="h-4 w-4 mr-2" />
							Export Report
							<ChevronDown className="h-4 w-4 ml-2" />
						</Button>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end">
						<DropdownMenuItem onClick={handleDownloadPDF}>
							<FileText className="h-4 w-4 mr-2" />
							Download PDF
						</DropdownMenuItem>
						<DropdownMenuItem onClick={handleDownloadText}>
							<FileText className="h-4 w-4 mr-2" />
							Download Text Report
						</DropdownMenuItem>
						<DropdownMenuItem onClick={handleDownloadCSV}>
							<TableIcon className="h-4 w-4 mr-2" />
							Download CSV Data
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			</div>

			{/* Score Overview Grid */}
			<div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
				<ScoreCard summary={summary} issues={issues} />
				<SiteHealthCard analysis={analysis} pages={pages} />
				<QuickStatsCard summary={summary} pages={pages} />
			</div>

			{/* Tabs */}
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
