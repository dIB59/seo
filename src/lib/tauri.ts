import { invoke } from "@tauri-apps/api/core"
import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult } from "./types"
import { fromPromise } from "@/src/lib/result";



export const execute = <T>(command: string, args?: Record<string, unknown>) =>
	fromPromise(invoke<T>(command, args));

export const startAnalysis = (url: string, settings: AnalysisSettingsRequest) =>
	execute<{ job_id: number }>("start_analysis", { url, settings });

export const getAllJobs = () =>
	execute<AnalysisProgress[]>("get_all_jobs");

export const getResult = (jobId: number) =>
	execute<CompleteAnalysisResult>("get_result", { jobId });

export const cancelAnalysis = (jobId: number) =>
	execute<void>("cancel_analysis", { jobId });

export const getAnalysisProgress = (jobId: number) =>
	execute<AnalysisProgress>("get_analysis_progress", { jobId });
