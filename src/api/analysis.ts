import { commands } from "@/src/bindings"
import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult, AnalysisJobResponse } from "@/src/lib/types"
import { Result } from "@/src/lib/result"

/**
 * Generic helper to wrap Tauri command results into our Result type.
 */
export async function wrapTauriCommand<T>(commandPromise: Promise<{ status: "ok" | "error"; data?: T; error?: unknown }>): Promise<Result<T, string>> {
    const res = await commandPromise;
    return res.status === "ok" ? Result.Ok(res.data as T) : Result.Err(res.error as string);
}

export const startAnalysis = async (url: string, settings: AnalysisSettingsRequest): Promise<Result<AnalysisJobResponse, string>> => {
    return wrapTauriCommand(commands.startAnalysis(url, settings || null));
}

export const getAllJobs = async (limit?: number, offset?: number): Promise<Result<AnalysisProgress[], string>> => {
    return wrapTauriCommand(commands.getAllJobs(limit ?? null, offset ?? null));
}

export const getPaginatedJobs = async (limit: number, offset: number, urlFilter?: string, statusFilter?: string) => {
    return wrapTauriCommand(commands.getPaginatedJobs(limit, offset, urlFilter ?? null, statusFilter ?? null));
}

export const getResult = async (jobId: string): Promise<Result<CompleteAnalysisResult, string>> => {
    return wrapTauriCommand(commands.getResult(jobId));
}

export const cancelAnalysis = async (jobId: string): Promise<Result<null, string>> => {
    return wrapTauriCommand(commands.cancelAnalysis(jobId));
}

export const getAnalysisProgress = async (jobId: string): Promise<Result<AnalysisProgress, string>> => {
    return wrapTauriCommand(commands.getAnalysisProgress(jobId));
}
