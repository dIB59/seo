import { invoke } from "@tauri-apps/api/core"
import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult } from "./types"


// Dynamic import of Tauri API
async function invokeTauri<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	return invoke<T>(command, args)
}

// Start a new analysis job
export async function startAnalysis(url: string, settings: AnalysisSettingsRequest): Promise<{ job_id: number }> {
	return invokeTauri("start_analysis", { url, settings })
}

// Get all analysis jobs
export async function getAllJobs(): Promise<AnalysisProgress[]> {
	const some = invokeTauri<AnalysisProgress[]>("get_all_jobs");
	return some;
}

// Get analysis result by job ID
export async function getResult(jobId: number): Promise<CompleteAnalysisResult> {
	let result = invokeTauri<CompleteAnalysisResult>("get_result", { jobId });
	if (!result) {
		throw new Error(`No results found for job ${jobId}`)
	}

	return result
}

// Cancel an analysis job
export async function cancelAnalysis(jobId: number): Promise<void> {
	return await invokeTauri("cancel_analysis", { jobId })
}

// Get analysis progress
export async function getAnalysisProgress(jobId: number): Promise<AnalysisProgress> {
	let job: Promise<AnalysisProgress> = invokeTauri("get_analysis_progress", { jobId })
	if (!job) {
		throw new Error(`Job ${jobId} not found`)
	}
	return job
}
