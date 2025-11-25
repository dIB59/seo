"use client"

import { useState } from "react"
import useSWR from "swr"
import { Search, RefreshCw, Info } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { UrlInputForm } from "@/src/components/url-input-form"
import { JobList } from "@/src/components/job-list"
import { AnalysisResults } from "@/src/components/analysis-results"
import { getAllJobs, startAnalysis, getResult, cancelAnalysis } from "@/src/lib/tauri"
import type { AnalysisSettingsRequest, CompleteAnalysisResult } from "@/src/lib/types"

export default function Home() {
	const [isLoading, setIsLoading] = useState(false)
	const [selectedResult, setSelectedResult] = useState<CompleteAnalysisResult | null>(null)
	const [error, setError] = useState<string | null>(null)

	const {
		data: jobs = [],
		mutate,
		isValidating,
	} = useSWR("jobs", getAllJobs, {
		refreshInterval: 10000,
	})

	const handleSubmit = async (url: string, settings: AnalysisSettingsRequest) => {
		setIsLoading(true)
		setError(null)
		try {
			await startAnalysis(url, settings)
			mutate()
		} catch (err) {
			setError(err instanceof Error ? err.message : "Failed to start analysis")
		} finally {
			setIsLoading(false)
		}
	}

	const handleViewResult = async (jobId: number) => {
		try {
			const result = await getResult(jobId)
			setSelectedResult(result)
		} catch (err) {
			setError(err instanceof Error ? err.message : "Failed to fetch results")
		}
	}

	const handleCancel = async (jobId: number) => {
		try {
			await cancelAnalysis(jobId)
			mutate()
		} catch (err) {
			setError(err instanceof Error ? err.message : "Failed to cancel analysis")
		}
	}

	if (selectedResult) {
		return (
			<main className="min-h-screen p-6 max-w-7xl mx-auto">
				<AnalysisResults result={selectedResult} onBack={() => setSelectedResult(null)} />
			</main>
		)
	}

	return (
		<main className="min-h-screen p-6 max-w-5xl mx-auto">
			{/* Header */}
			<div className="flex items-center justify-between mb-8">
				<div className="flex items-center gap-3">
					<div className="p-2 bg-primary/20 rounded-lg">
						<Search className="h-6 w-6 text-primary" />
					</div>
					<div>
						<h1 className="text-2xl font-bold">SEO Analyzer</h1>
						<p className="text-sm text-muted-foreground">Analyze websites for SEO issues and recommendations</p>
					</div>
				</div>
				<Button variant="ghost" size="sm" onClick={() => mutate()} disabled={isValidating}>
					<RefreshCw className={`h-4 w-4 mr-2 ${isValidating ? "animate-spin" : ""}`} />
					Refresh
				</Button>
			</div>

			{/* Error Message */}
			{error && (
				<div className="mb-6 p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
					<p className="text-sm text-destructive">{error}</p>
				</div>
			)}

			{/* URL Input Form */}
			<div className="mb-8">
				<UrlInputForm onSubmit={handleSubmit} isLoading={isLoading} />
			</div>

			{/* Analysis Jobs */}
			<div>
				<div className="flex items-center justify-between mb-4">
					<h2 className="text-lg font-semibold">Analysis Jobs</h2>
					{jobs.length > 0 && <span className="text-sm text-muted-foreground">{jobs.length} jobs</span>}
				</div>
				<JobList jobs={jobs} onViewResult={handleViewResult} onCancel={handleCancel} />
			</div>
		</main>
	)
}
