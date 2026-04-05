import { commands } from "@/src/bindings";
import type {
  AnalysisJobResponse,
  AnalysisProgress,
  AnalysisSettingsRequest,
  CompleteAnalysisResponse,
  HeadingElement,
  ImageElement,
  JobStatus,
  JsonValue,
  LinkDetail,
  PageAnalysisData,
  SeoIssue,
} from "@/src/bindings";
import { Result } from "@/src/lib/result";

export type {
  AnalysisJobResponse,
  AnalysisProgress,
  AnalysisSettingsRequest,
  CompleteAnalysisResponse,
  HeadingElement,
  ImageElement,
  JobStatus,
  JsonValue,
  LinkDetail,
  PageAnalysisData,
  SeoIssue,
};

export async function wrapTauriCommand<T>(
  commandPromise: Promise<{ status: "ok" | "error"; data?: T; error?: unknown }>,
): Promise<Result<T, string>> {
  const res = await commandPromise;
  return res.status === "ok"
    ? Result.Ok(res.data as T)
    : Result.Err(String(res.error ?? "Unknown error"));
}

export async function startAnalysis(
  url: string,
  settings: AnalysisSettingsRequest,
): Promise<Result<AnalysisJobResponse, string>> {
  return wrapTauriCommand(commands.startAnalysis(url, settings || null));
}

export async function getAllJobs(
  limit?: number,
  offset?: number,
): Promise<Result<AnalysisProgress[], string>> {
  return wrapTauriCommand(commands.getAllJobs(limit ?? null, offset ?? null));
}

export async function getPaginatedJobs(
  limit: number,
  offset: number,
  urlFilter?: string,
  statusFilter?: string,
) {
  return wrapTauriCommand(
    commands.getPaginatedJobs(limit, offset, urlFilter ?? null, statusFilter ?? null),
  );
}

export async function getResult(jobId: string): Promise<Result<CompleteAnalysisResponse, string>> {
  return wrapTauriCommand(commands.getResult(jobId));
}

export async function cancelAnalysis(jobId: string): Promise<Result<null, string>> {
  return wrapTauriCommand(commands.cancelAnalysis(jobId));
}

export async function getAnalysisProgress(
  jobId: string,
): Promise<Result<AnalysisProgress, string>> {
  return wrapTauriCommand(commands.getAnalysisProgress(jobId));
}

export async function getAnalysisDefaults(): Promise<Result<AnalysisSettingsRequest, string>> {
  return wrapTauriCommand(commands.getAnalysisDefaults());
}

export async function getFreeTierDefaults(): Promise<Result<AnalysisSettingsRequest, string>> {
  return wrapTauriCommand(commands.getFreeTierDefaults());
}
