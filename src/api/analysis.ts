import { execute } from "@/src/lib/tauri"
import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult } from "@/src/lib/types"

export const startAnalysis = (url: string, settings: AnalysisSettingsRequest) =>
    execute<{ job_id: string }>("start_analysis", { url, settings });

export const getAllJobs = () =>
    execute<AnalysisProgress[]>("get_all_jobs");

export const getResult = (jobId: string) =>
    execute<CompleteAnalysisResult>("get_result", { jobId });

export const cancelAnalysis = (jobId: string) =>
    execute<void>("cancel_analysis", { jobId });

export const getAnalysisProgress = (jobId: string) =>
    execute<AnalysisProgress>("get_analysis_progress", { jobId });
