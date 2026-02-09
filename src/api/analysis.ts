import { commands } from "@/src/bindings"
import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult, AnalysisJobResponse } from "@/src/lib/types"
import { Result } from "@/src/lib/result"

export const startAnalysis = async (url: string, settings: AnalysisSettingsRequest): Promise<Result<AnalysisJobResponse, string>> => {
    const res = await commands.startAnalysis(url, settings || null)
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error as string)
}

export const getAllJobs = async (): Promise<Result<AnalysisProgress[], string>> => {
    const res = await commands.getAllJobs()
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error as string)
}

export const getResult = async (jobId: string): Promise<Result<CompleteAnalysisResult, string>> => {
    const res = await commands.getResult(jobId)
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error as string)
}

export const cancelAnalysis = async (jobId: string): Promise<Result<null, string>> => {
    const res = await commands.cancelAnalysis(jobId)
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error as string)
}

export const getAnalysisProgress = async (jobId: string): Promise<Result<AnalysisProgress, string>> => {
    const res = await commands.getAnalysisProgress(jobId)
    return res.status === "ok" ? Result.Ok(res.data) : Result.Err(res.error as string)
}
