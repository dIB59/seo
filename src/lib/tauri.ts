import { invoke } from "@tauri-apps/api/core"
import type { AnalysisProgress, AnalysisSettingsRequest, CompleteAnalysisResult } from "./types"
import { fromPromise } from "@/src/lib/result";


export const startAnalysis = (url: string, settings: AnalysisSettingsRequest) =>
	fromPromise(invoke<{ job_id: number }>("start_analysis", { url, settings }));

export const getAllJobs = () =>
	fromPromise(invoke<AnalysisProgress[]>("get_all_jobs"));

export const getResult = (jobId: number) =>
	fromPromise(invoke<CompleteAnalysisResult>("get_result", { jobId }));

export const cancelAnalysis = (jobId: number) => {

	const some = invoke<void>("cancel_analysis", { jobId });
	return fromPromise(some)
}

export const getAnalysisProgress = (jobId: number) =>
	fromPromise(invoke<AnalysisProgress>("get_analysis_progress", { jobId }));
