"use client"

import type React from "react"
import { useState } from "react"
import {
	ArrowLeft,
	FileText,
	AlertTriangle,
	Clock,
	CheckCircle2,
	XCircle,
	AlertCircle,
	Lightbulb,
	Download,
	ExternalLink,
	Smartphone,
	ImageIcon,
	Link2,
	FileCode,
	Eye,
	Shield,
	Zap,
	Search,
	BarChart3,
	ChevronRight,
} from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/src/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/src/components/ui/tabs"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/src/components/ui/accordion"
import { Progress } from "@/src/components/ui/progress"
import { Badge } from "@/src/components/ui/badge"
import { Separator } from "@/src/components/ui/separator"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/src/components/ui/dialog"
import { cn } from "@/src/lib/utils"
import type { CompleteAnalysisResult, SeoIssue, PageAnalysisData } from "@/src/lib/types"

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function getScoreColor(score: number) {
	if (score >= 80) return "text-success"
	if (score >= 50) return "text-warning"
	return "text-destructive"
}

function getScoreBgColor(score: number) {
	if (score >= 80) return "bg-success"
	if (score >= 50) return "bg-warning"
	return "bg-destructive"
}

function getScoreLabel(score: number) {
	if (score >= 90) return "Excellent"
	if (score >= 80) return "Good"
	if (score >= 60) return "Fair"
	if (score >= 40) return "Poor"
	return "Critical"
}

function getLoadTimeColor(time: number) {
	if (time < 1) return "text-success"
	if (time < 2) return "text-warning"
	return "text-destructive"
}

// ============================================================================
// COMPOSABLE UI COMPONENTS
// ============================================================================

function IssueIcon({ type }: { type: string }) {
	const iconMap: Record<string, React.ReactNode> = {
		Critical: <XCircle className="h-4 w-4 text-destructive" />,
		Warning: <AlertCircle className="h-4 w-4 text-warning" />,
		Suggestion: <Lightbulb className="h-4 w-4 text-primary" />,
	}
	return iconMap[type] ?? <AlertTriangle className="h-4 w-4 text-muted-foreground" />
}

function IssueBadge({ type, count }: { type: string; count?: number }) {
	const label = count !== undefined ? `${count} ${type}` : type
	const variants: Record<string, string> = {
		Critical: "bg-destructive/15 text-destructive border-destructive/20",
		Warning: "bg-warning/15 text-warning border-warning/20",
		Suggestion: "bg-primary/15 text-primary border-primary/20",
	}
	return (
		<Badge variant="outline" className={cn("text-xs font-medium", variants[type])}>
			{label}
		</Badge>
	)
}

function ScoreRing({ score, size = "lg", label }: { score: number; size?: "sm" | "md" | "lg"; label?: string }) {
	const dimensions = { sm: 48, md: 64, lg: 80 }
	const strokeWidth = { sm: 5, md: 6, lg: 8 }
	const fontSize = { sm: "text-sm", md: "text-lg", lg: "text-xl" }

	const dim = dimensions[size]
	const stroke = strokeWidth[size]
	const radius = (dim - stroke) / 2
	const circumference = 2 * Math.PI * radius

	return (
		<div className="relative inline-flex items-center justify-center shrink-0">
			<svg className="transform -rotate-90" width={dim} height={dim}>
				<circle
					cx={dim / 2}
					cy={dim / 2}
					r={radius}
					strokeWidth={stroke}
					stroke="currentColor"
					fill="none"
					className="text-muted/30"
				/>
				<circle
					cx={dim / 2}
					cy={dim / 2}
					r={radius}
					strokeWidth={stroke}
					stroke="currentColor"
					fill="none"
					strokeDasharray={circumference}
					strokeDashoffset={circumference - (circumference * score) / 100}
					strokeLinecap="round"
					className={getScoreBgColor(score)}
				/>
			</svg>
			<div className="absolute inset-0 flex flex-col items-center justify-center">
				<span className={cn("font-bold", getScoreColor(score), fontSize[size])}>{score}</span>
				{label && <span className="text-[10px] text-muted-foreground">{label}</span>}
			</div>
		</div>
	)
}

function StatusIcon({
	active,
	activeIcon: ActiveIcon,
	inactiveIcon: InactiveIcon,
}: {
	active: boolean
	activeIcon?: React.ComponentType<{ className?: string }>
	inactiveIcon?: React.ComponentType<{ className?: string }>
}) {
	const Icon = active ? ActiveIcon || CheckCircle2 : InactiveIcon || XCircle
	return <Icon className={cn("h-5 w-5", active ? "text-success" : "text-muted-foreground")} />
}

function StatItem({
	icon: Icon,
	label,
	value,
	subValue,
}: {
	icon: React.ComponentType<{ className?: string }>
	label: string
	value: React.ReactNode
	subValue?: string
}) {
	return (
		<div className="flex items-center gap-2 p-2 rounded-lg bg-muted/50">
			<Icon className="h-4 w-4 text-muted-foreground shrink-0" />
			<div className="min-w-0 flex-1">
				<p className="text-sm font-semibold truncate">{value}</p>
				<p className="text-[10px] text-muted-foreground truncate">{label}</p>
				{subValue && <p className="text-[10px] text-muted-foreground">{subValue}</p>}
			</div>
		</div>
	)
}

function HealthIndicator({ label, active }: { label: string; active: boolean }) {
	return (
		<div className="flex flex-col items-center gap-1 p-2 rounded-lg bg-muted/50">
			<StatusIcon active={active} activeIcon={CheckCircle2} inactiveIcon={AlertCircle} />
			<span className="text-xs text-muted-foreground">{label}</span>
		</div>
	)
}

function MetricBadge({ label, value }: { label: string; value: number | string }) {
	return (
		<div className="text-center p-2 rounded-lg bg-muted/50">
			<p className="text-lg font-semibold">{value}</p>
			<p className="text-[10px] text-muted-foreground">{label}</p>
		</div>
	)
}

function ProgressRow({
	label,
	value,
	total,
	color,
}: {
	label: string
	value: number
	total: number
	color: "success" | "warning" | "destructive" | "primary"
}) {
	const colorMap = {
		success: "[&>div]:bg-success",
		warning: "[&>div]:bg-warning",
		destructive: "[&>div]:bg-destructive",
		primary: "[&>div]:bg-primary",
	}
	return (
		<div className="flex items-center gap-3">
			<div className="w-20 text-sm text-muted-foreground">{label}</div>
			<Progress value={total > 0 ? (value / total) * 100 : 0} className={cn("flex-1 h-2", colorMap[color])} />
			<div className="w-8 text-sm text-right">{value}</div>
		</div>
	)
}

// ============================================================================
// LIGHTHOUSE SCORES COMPONENT
// ============================================================================

function LighthouseScores({ page }: { page: PageAnalysisData }) {
	if (!page.lighthouse_performance) return null

	const scores = [
		{ label: "Performance", value: page.lighthouse_performance, icon: Zap },
		{ label: "Accessibility", value: page.lighthouse_accessibility, icon: Eye },
		{ label: "Best Practices", value: page.lighthouse_best_practices, icon: Shield },
		{ label: "SEO", value: page.lighthouse_seo, icon: Search },
	]

	return (
		<div className="grid grid-cols-4 gap-2">
			{scores.map((score) => (
				<div key={score.label} className="flex flex-col items-center gap-1 p-2 rounded-lg bg-muted/50">
					<ScoreRing score={score.value || 0} size="sm" />
					<span className="text-[10px] text-muted-foreground text-center">{score.label}</span>
				</div>
			))}
		</div>
	)
}


const isBroken = (p: PageAnalysisData) => p.status_code! >= 400;

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


function BrokenPageRow({ page, onClick }: { page: PageAnalysisData; onClick: () => void }) {
	return (
		<TableRow className="cursor-pointer bg-destructive/5 hover:bg-destructive/10 text-destructive " onClick={onClick}>
			<TableCell className="max-w-[200px]">
				<div className="flex flex-col gap-0.5">
					<span className="font-medium text-sm truncate text-foreground">
						{page.url.replace(/^https?:\/\/[^/]+/, "") || "/"}
					</span>
					<span className="text-xs text-muted-foreground truncate">
						{page.title || "No title"}
					</span>
				</div>
			</TableCell>

			{/* ----- red status code ----- */}
			<TableCell className="text-center">
				<span className="text-destructive font-medium">{page.load_time.toPrecision(2) + "s"}</span>
			</TableCell>

			{/* ----- same data as healthy row ----- */}
			<TableCell className="text-center text-destructive ">-</TableCell>

			<TableCell className="text-center text-xs text-destructive ">
				–
			</TableCell>

			<TableCell className="text-center text-destructive ">
				<div className="flex items-center justify-center gap-1">
					<span>{page.image_count}</span>
					{page.images_without_alt > 0 && (
						<span className="text-destructive/80 text-xs">(-{page.images_without_alt})</span>
					)}
				</div>
			</TableCell>

			<TableCell className="text-center text-xs">
				{page.internal_links}/{page.external_links}
			</TableCell>

			<TableCell className="text-center">
				<div className="flex items-center justify-center gap-1.5">
					-
				</div>
			</TableCell>

			<TableCell className="text-center">
				{page.lighthouse_seo ? (
					<span className={cn("text-sm font-medium", getScoreColor(page.lighthouse_seo))}>
						{page.lighthouse_seo}
					</span>
				) : (
					<span className="text-muted-foreground">-</span>
				)}
			</TableCell>

			<TableCell>
				<ChevronRight className="h-4 w-4 text-muted-foreground" />
			</TableCell>
		</TableRow>
	);
}



function HealthyPageRow({ page, onClick }: { page: PageAnalysisData; onClick: () => void }) {
	return (
		<TableRow className="cursor-pointer hover:bg-muted/50" onClick={onClick}>
			<TableCell className="max-w-[200px]">
				<div className="flex flex-col gap-0.5">
					<span className="font-medium text-sm truncate">{page.url.replace(/^https?:\/\/[^/]+/, "") || "/"}</span>
					<span className="text-xs text-muted-foreground truncate">{page.title || "No title"}</span>
				</div>
			</TableCell>
			<TableCell className="text-center">
				<span className={cn("font-medium", getLoadTimeColor(page.load_time))}>{page.load_time.toFixed(2)}s</span>
			</TableCell>
			<TableCell className="text-center">{page.word_count.toLocaleString()}</TableCell>
			<TableCell className="text-center">
				<span className="text-xs">
					{page.h1_count}/{page.h2_count}/{page.h3_count}
				</span>
			</TableCell>
			<TableCell className="text-center">
				<div className="flex items-center justify-center gap-1">
					<span>{page.image_count}</span>
					{page.images_without_alt > 0 && (
						<span className="text-destructive/80 text-xs">(-{page.images_without_alt})</span>
					)}
				</div>
			</TableCell>
			<TableCell className="text-center">
				<span className="text-xs">
					{page.internal_links}/{page.external_links}
				</span>
			</TableCell>
			<TableCell className="text-center">
				<div className="flex items-center justify-center gap-1.5">
					<Smartphone className={cn("h-3.5 w-3.5", page.mobile_friendly ? "text-success" : "text-muted-foreground")} />
					<FileCode
						className={cn("h-3.5 w-3.5", page.has_structured_data ? "text-success" : "text-muted-foreground")}
					/>
				</div>
			</TableCell>
			<TableCell className="text-center">
				{page.lighthouse_seo ? (
					<span className={cn("text-sm font-medium", getScoreColor(page.lighthouse_seo))}>{page.lighthouse_seo}</span>
				) : (
					<span className="text-muted-foreground">-</span>
				)}
			</TableCell>
			<TableCell>
				<ChevronRight className="h-4 w-4 text-muted-foreground" />
			</TableCell>
		</TableRow>
	)
}

// ============================================================================
// PAGE DETAIL MODAL
// ============================================================================

function PageDetailModal({
	page,
	open,
	onClose,
}: { page: PageAnalysisData | null; open: boolean; onClose: () => void }) {
	if (!page) return null

	return (
		<Dialog open={open} onOpenChange={onClose}>
			<DialogContent className="max-w-3xl max-h-[90vh] overflow-auto">
				<DialogHeader>
					<DialogTitle className="truncate pr-8">{page.url}</DialogTitle>
					<p className="text-sm text-muted-foreground truncate">{page.title || "No title"}</p>
				</DialogHeader>

				<div className="space-y-6">
					<LighthouseScores page={page} />

					<Separator />

					<div>
						<h4 className="text-sm font-medium mb-3">Meta Information</h4>
						<div className="space-y-2 text-sm">
							{[
								{ label: "Title", value: page.title },
								{ label: "Description", value: page.meta_description },
								{ label: "Keywords", value: page.meta_keywords },
								{ label: "Canonical", value: page.canonical_url },
							].map(({ label, value }) => (
								<div key={label} className="grid grid-cols-[100px_1fr] gap-2">
									<span className="text-muted-foreground">{label}:</span>
									<span className="truncate">{value || <span className="text-muted-foreground">None</span>}</span>
								</div>
							))}
						</div>
					</div>

					<Separator />

					<div>
						<h4 className="text-sm font-medium mb-3">Content Metrics</h4>
						<div className="grid grid-cols-2 md:grid-cols-4 gap-3">
							<StatItem icon={FileText} label="Word Count" value={page.word_count.toLocaleString()} />
							<StatItem icon={Clock} label="Load Time" value={`${page.load_time.toFixed(2)}s`} />
							<StatItem icon={FileCode} label="Content Size" value={`${(page.content_size / 1024).toFixed(1)}KB`} />
							<StatItem icon={BarChart3} label="Status Code" value={page.status_code || "N/A"} />
						</div>
					</div>

					<Separator />

					<div>
						<h4 className="text-sm font-medium mb-3">Page Structure</h4>
						<div className="grid grid-cols-3 md:grid-cols-6 gap-3">
							<MetricBadge label="H1 Tags" value={page.h1_count} />
							<MetricBadge label="H2 Tags" value={page.h2_count} />
							<MetricBadge label="H3 Tags" value={page.h3_count} />
							<MetricBadge label="Images" value={page.image_count} />
							<MetricBadge label="Int. Links" value={page.internal_links} />
							<MetricBadge label="Ext. Links" value={page.external_links} />
						</div>
					</div>

					<Separator />

					<div className="flex flex-wrap gap-2">
						<Badge
							variant="outline"
							className={cn(
								"text-xs",
								page.mobile_friendly ? "bg-success/15 text-success border-success/20" : "bg-muted",
							)}
						>
							<Smartphone className="h-3 w-3 mr-1" />
							{page.mobile_friendly ? "Mobile Friendly" : "Not Mobile Friendly"}
						</Badge>
						<Badge
							variant="outline"
							className={cn(
								"text-xs",
								page.has_structured_data ? "bg-success/15 text-success border-success/20" : "bg-muted",
							)}
						>
							<FileCode className="h-3 w-3 mr-1" />
							{page.has_structured_data ? "Has Structured Data" : "No Structured Data"}
						</Badge>
						{page.images_without_alt > 0 && (
							<Badge variant="outline" className="text-xs bg-destructive/15 text-destructive border-destructive/20">
								<ImageIcon className="h-3 w-3 mr-1" />
								{page.images_without_alt} Images Missing Alt
							</Badge>
						)}
					</div>
				</div>
			</DialogContent>
		</Dialog>
	)
}

// ============================================================================
// REPORT GENERATOR
// ============================================================================

function generateReport(result: CompleteAnalysisResult): string {
	const { analysis, summary, pages, issues } = result
	const criticalIssues = issues.filter((i) => i.issue_type === "Critical")
	const warningIssues = issues.filter((i) => i.issue_type === "Warning")
	const suggestionIssues = issues.filter((i) => i.issue_type === "Suggestion")

	return `
SEO ANALYSIS REPORT
${"=".repeat(60)}

Website: ${analysis.url}
Generated: ${new Date().toLocaleString()}
Analysis Completed: ${analysis.completed_at ? new Date(analysis.completed_at).toLocaleString() : "N/A"}

${"=".repeat(60)}
EXECUTIVE SUMMARY
${"=".repeat(60)}

Overall SEO Score: ${summary.seo_score}/100 (${getScoreLabel(summary.seo_score)})
Pages Analyzed: ${pages.length}
Total Issues Found: ${summary.total_issues}
  - Critical: ${criticalIssues.length}
  - Warnings: ${warningIssues.length}
  - Suggestions: ${suggestionIssues.length}

Average Load Time: ${summary.avg_load_time.toFixed(2)}s
Total Word Count: ${summary.total_words.toLocaleString()}

Site Health:
  - SSL Certificate: ${analysis.ssl_certificate ? "Valid" : "Missing"}
  - Sitemap: ${analysis.sitemap_found ? "Found" : "Not Found"}
  - robots.txt: ${analysis.robots_txt_found ? "Found" : "Not Found"}

${"=".repeat(60)}
CRITICAL ISSUES (${criticalIssues.length})
${"=".repeat(60)}
${criticalIssues.length === 0
			? "\nNo critical issues found.\n"
			: criticalIssues
				.map(
					(issue, i) => `
${i + 1}. ${issue.title}
   Page: ${issue.page_url}
   Description: ${issue.description}
   Recommendation: ${issue.recommendation}
`,
				)
				.join("")
		}

${"=".repeat(60)}
WARNINGS (${warningIssues.length})
${"=".repeat(60)}
${warningIssues.length === 0
			? "\nNo warnings found.\n"
			: warningIssues
				.map(
					(issue, i) => `
${i + 1}. ${issue.title}
   Page: ${issue.page_url}
   Description: ${issue.description}
   Recommendation: ${issue.recommendation}
`,
				)
				.join("")
		}

${"=".repeat(60)}
SUGGESTIONS (${suggestionIssues.length})
${"=".repeat(60)}
${suggestionIssues.length === 0
			? "\nNo suggestions.\n"
			: suggestionIssues
				.map(
					(issue, i) => `
${i + 1}. ${issue.title}
   Page: ${issue.page_url}
   Description: ${issue.description}
   Recommendation: ${issue.recommendation}
`,
				)
				.join("")
		}

${"=".repeat(60)}
PAGE-BY-PAGE ANALYSIS
${"=".repeat(60)}
${pages
			.map(
				(page, i) => `
${i + 1}. ${page.url}
   Title: ${page.title || "Missing"}
   Meta Description: ${page.meta_description ? "Present" : "Missing"}
   Load Time: ${page.load_time.toFixed(2)}s
   Word Count: ${page.word_count}
   Headings: H1(${page.h1_count}) H2(${page.h2_count}) H3(${page.h3_count})
   Images: ${page.image_count} (${page.images_without_alt} missing alt)
   Links: ${page.internal_links} internal, ${page.external_links} external
   Mobile Friendly: ${page.mobile_friendly ? "Yes" : "No"}
   Structured Data: ${page.has_structured_data ? "Yes" : "No"}
   ${page.lighthouse_seo ? `Lighthouse SEO: ${page.lighthouse_seo}/100` : ""}
`,
			)
			.join("")}

${"=".repeat(60)}
END OF REPORT
${"=".repeat(60)}
`.trim()
}

// ============================================================================
// CARD SECTIONS
// ============================================================================

function ScoreCard({
	summary,
	issues,
}: {
	summary: CompleteAnalysisResult["summary"]
	issues: SeoIssue[]
}) {
	const criticalCount = issues.filter((i) => i.issue_type === "Critical").length
	const warningCount = issues.filter((i) => i.issue_type === "Warning").length
	const suggestionCount = issues.filter((i) => i.issue_type === "Suggestion").length

	return (
		<Card>
			<CardContent className="p-6">
				<div className="flex items-start gap-6">
					<ScoreRing score={summary.seo_score} size="lg" />
					<div className="flex-1 min-w-0 space-y-2">
						<div>
							<h3 className="text-lg font-semibold">SEO Score</h3>
							<p className={cn("text-sm font-medium", getScoreColor(summary.seo_score))}>
								{getScoreLabel(summary.seo_score)}
							</p>
						</div>
						<Separator />
						<div className="pt-1">
							<p className="text-xs text-muted-foreground mb-2">Issues Found</p>
							<div className="flex flex-wrap gap-2">
								<IssueBadge type="Critical" count={criticalCount} />
								<IssueBadge type="Warning" count={warningCount} />
								<IssueBadge type="Suggestion" count={suggestionCount} />
							</div>
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	)
}

function SiteHealthCard({
	analysis,
	pages,
}: {
	analysis: CompleteAnalysisResult["analysis"]
	pages: PageAnalysisData[]
}) {
	const mobilePages = pages.filter((p) => p.mobile_friendly).length
	const structuredDataPages = pages.filter((p) => p.has_structured_data).length

	return (
		<Card>
			<CardContent className="p-6">
				<h3 className="text-sm font-semibold mb-3">Site Health</h3>
				<div className="grid grid-cols-3 gap-3">
					<HealthIndicator label="SSL" active={analysis.ssl_certificate} />
					<HealthIndicator label="Sitemap" active={analysis.sitemap_found} />
					<HealthIndicator label="robots.txt" active={analysis.robots_txt_found} />
				</div>
				<Separator className="my-3" />
				<div className="grid grid-cols-2 gap-2 text-sm">
					<div className="flex justify-between">
						<span className="text-muted-foreground">Mobile Ready</span>
						<span className={mobilePages === pages.length ? "text-success" : "text-warning"}>
							{mobilePages}/{pages.length}
						</span>
					</div>
					<div className="flex justify-between">
						<span className="text-muted-foreground">Structured Data</span>
						<span className={structuredDataPages === pages.length ? "text-success" : "text-muted-foreground"}>
							{structuredDataPages}/{pages.length}
						</span>
					</div>
				</div>
			</CardContent>
		</Card>
	)
}

function QuickStatsCard({
	summary,
	pages,
}: {
	summary: CompleteAnalysisResult["summary"]
	pages: PageAnalysisData[]
}) {
	const totalImages = pages.reduce((acc, p) => acc + p.image_count, 0)
	const totalMissingAlt = pages.reduce((acc, p) => acc + p.images_without_alt, 0)
	const totalInternalLinks = pages.reduce((acc, p) => acc + p.internal_links, 0)

	return (
		<Card>
			<CardContent className="p-6">
				<h3 className="text-sm font-semibold mb-3">Quick Stats</h3>
				<div className="grid grid-cols-2 gap-3">
					<StatItem icon={Clock} label="Avg Load Time" value={`${summary.avg_load_time.toFixed(2)}s`} />
					<StatItem icon={FileText} label="Total Words" value={summary.total_words.toLocaleString()} />
					<StatItem
						icon={ImageIcon}
						label="Images"
						value={
							<span>
								{totalImages}
								{totalMissingAlt > 0 && (
									<span className="text-destructive/80 text-xs ml-1">({totalMissingAlt} no alt)</span>
								)}
							</span>
						}
					/>
					<StatItem icon={Link2} label="Internal Links" value={totalInternalLinks} />
				</div>
			</CardContent>
		</Card>
	)
}

// ============================================================================
// TABS CONTENT
// ============================================================================

function IssuesTab({ issues }: { issues: SeoIssue[] }) {
	const groupedIssues: Record<string, SeoIssue[]> = {}
	issues.forEach((issue) => {
		if (!groupedIssues[issue.title]) groupedIssues[issue.title] = []
		groupedIssues[issue.title].push(issue)
	})

	if (Object.keys(groupedIssues).length === 0) {
		return (
			<Card>
				<CardContent className="p-6 text-center">
					<CheckCircle2 className="h-12 w-12 text-success mx-auto mb-2" />
					<p className="text-muted-foreground">No issues found. Great job!</p>
				</CardContent>
			</Card>
		)
	}

	return (
		<Accordion type="multiple" className="space-y-2">
			{Object.entries(groupedIssues).map(([title, issueGroup]) => (
				<AccordionItem key={title} value={title} className="border rounded-lg px-4">
					<AccordionTrigger className="hover:no-underline">
						<div className="flex items-center gap-3">
							<IssueIcon type={issueGroup[0].issue_type} />
							<span className="font-medium">{title}</span>
							<IssueBadge type={issueGroup[0].issue_type} />
							<span className="text-xs text-muted-foreground">
								{issueGroup.length} {issueGroup.length === 1 ? "page" : "pages"}
							</span>
						</div>
					</AccordionTrigger>
					<AccordionContent>
						<div className="space-y-3 pt-2">
							<p className="text-sm text-muted-foreground">{issueGroup[0].description}</p>
							<div className="p-3 bg-muted/50 rounded-lg">
								<p className="text-sm font-medium mb-1">Recommendation</p>
								<p className="text-sm text-muted-foreground">{issueGroup[0].recommendation}</p>
							</div>
							<div className="space-y-1">
								<p className="text-xs font-medium text-muted-foreground">Affected Pages:</p>
								{issueGroup.map((issue, idx) => (
									<p key={idx} className="text-xs text-muted-foreground truncate">
										{issue.page_url}
									</p>
								))}
							</div>
						</div>
					</AccordionContent>
				</AccordionItem>
			))}
		</Accordion>
	)
}

function PagesTab({
	pages,
	onSelectPage,
}: { pages: PageAnalysisData[]; onSelectPage: (page: PageAnalysisData) => void }) {
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
					{pages.map((page, idx) => (
						<PageDetailRow key={idx} page={page} onClick={() => onSelectPage(page)} />
					))}
				</TableBody>
			</Table>
		</Card>
	)
}

function OverviewTab({ issues, pages }: { issues: SeoIssue[]; pages: PageAnalysisData[] }) {
	const criticalCount = issues.filter((i) => i.issue_type === "Critical").length
	const warningCount = issues.filter((i) => i.issue_type === "Warning").length
	const suggestionCount = issues.filter((i) => i.issue_type === "Suggestion").length

	const fastPages = pages.filter((p) => p.load_time < 1).length
	const mediumPages = pages.filter((p) => p.load_time >= 1 && p.load_time < 2).length
	const slowPages = pages.filter((p) => p.load_time >= 2).length

	return (
		<div className="grid grid-cols-1 md:grid-cols-2 gap-4">
			<Card>
				<CardHeader>
					<CardTitle className="text-sm">Issue Distribution</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="space-y-3">
						<ProgressRow label="Critical" value={criticalCount} total={issues.length} color="destructive" />
						<ProgressRow label="Warning" value={warningCount} total={issues.length} color="warning" />
						<ProgressRow label="Suggestion" value={suggestionCount} total={issues.length} color="primary" />
					</div>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle className="text-sm">Performance Summary</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="space-y-3">
						<ProgressRow label="Fast (<1s)" value={fastPages} total={pages.length} color="success" />
						<ProgressRow label="Medium (1-2s)" value={mediumPages} total={pages.length} color="warning" />
						<ProgressRow label="Slow (>2s)" value={slowPages} total={pages.length} color="destructive" />
					</div>
				</CardContent>
			</Card>
		</div>
	)
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

interface AnalysisResultsProps {
	result: CompleteAnalysisResult
	onBack: () => void
}

export function AnalysisResults({ result, onBack }: AnalysisResultsProps) {
	const { analysis, pages, issues, summary } = result
	const [selectedPage, setSelectedPage] = useState<PageAnalysisData | null>(null)

	const handleGenerateReport = () => {
		const reportText = generateReport(result)
		const blob = new Blob([reportText], { type: "text/plain" })
		const url = URL.createObjectURL(blob)
		const a = document.createElement("a")
		a.href = url
		a.download = `seo-report-${analysis.url.replace(/https?:\/\//, "").replace(/[^a-z0-9]/gi, "-")}-${new Date().toISOString().split("T")[0]}.txt`
		document.body.appendChild(a)
		a.click()
		document.body.removeChild(a)
		URL.revokeObjectURL(url)
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
				<Button variant="outline" onClick={handleGenerateReport} className="shrink-0 bg-transparent">
					<Download className="h-4 w-4 mr-2" />
					Generate Report
				</Button>
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
					<TabsTrigger value="overview">Overview</TabsTrigger>
				</TabsList>

				<TabsContent value="issues" className="mt-4">
					<IssuesTab issues={issues} />
				</TabsContent>

				<TabsContent value="pages" className="mt-4">
					<PagesTab pages={pages} onSelectPage={setSelectedPage} />
				</TabsContent>

				<TabsContent value="overview" className="mt-4">
					<OverviewTab issues={issues} pages={pages} />
				</TabsContent>
			</Tabs>

			{/* Page Detail Modal */}
			<PageDetailModal page={selectedPage} open={!!selectedPage} onClose={() => setSelectedPage(null)} />
		</div>
	)
}
