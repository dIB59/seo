import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult } from "./types"
import { mockJobs, mockResults } from "./mock-data"

// Check if running in Tauri environment
function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window
}

// Dynamic import of Tauri API
async function invokeTauri<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core")
  return invoke<T>(command, args)
}

// Start a new analysis job
export async function startAnalysis(url: string, settings: AnalysisSettingsRequest): Promise<{ job_id: number }> {
  if (isTauri()) {
    return invokeTauri("start_analysis", { url, settings })
  }
  // Mock: create a new job
  const newJobId = Math.max(...mockJobs.map((j) => j.job_id), 0) + 1
  mockJobs.unshift({
    job_id: newJobId,
    url,
    job_status: "queued",
    result_id: null,
    analysis_status: null,
    progress: null,
    analyzed_pages: null,
    total_pages: null,
  })
  return { job_id: newJobId }
}

// Get all analysis jobs
export async function getAllJobs(): Promise<AnalysisProgress[]> {
  if (isTauri()) {
    return invokeTauri("get_all_jobs")
  }
  return mockJobs
}

// Get analysis result by job ID
export async function getResult(jobId: number): Promise<CompleteAnalysisResult> {
  if (isTauri()) {
    return invokeTauri("get_result", { jobId })
  }
  const result = mockResults[jobId]
  if (!result) {
    throw new Error(`No results found for job ${jobId}`)
  }
  return result
}

// Cancel an analysis job
export async function cancelAnalysis(jobId: number): Promise<void> {
  if (isTauri()) {
    await invokeTauri("cancel_analysis", { jobId })
    return
  }
  // Mock: remove from jobs list
  const index = mockJobs.findIndex((j) => j.job_id === jobId)
  if (index !== -1) {
    mockJobs.splice(index, 1)
  }
}

// Get analysis progress
export async function getAnalysisProgress(jobId: number): Promise<AnalysisProgress> {
  if (isTauri()) {
    return invokeTauri("get_analysis_progress", { jobId })
  }
  const job = mockJobs.find((j) => j.job_id === jobId)
  if (!job) {
    throw new Error(`Job ${jobId} not found`)
  }
  return job
}
