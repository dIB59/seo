import { commands } from "@/src/bindings";
import type {
  ReportPattern,
  ReportPatternParams,
  ReportData,
} from "@/src/bindings";

export type { ReportPattern, ReportPatternParams, ReportData };

export async function listReportPatterns(): Promise<ReportPattern[]> {
  const res = await commands.listReportPatterns();
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to list report patterns");
}

export async function createReportPattern(params: ReportPatternParams): Promise<ReportPattern> {
  const res = await commands.createReportPattern(params);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to create report pattern");
}

export async function updateReportPattern(id: string, params: ReportPatternParams): Promise<ReportPattern> {
  const res = await commands.updateReportPattern(id, params);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to update report pattern");
}

export async function toggleReportPattern(id: string, enabled: boolean): Promise<void> {
  const res = await commands.toggleReportPattern(id, enabled);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to toggle report pattern");
}

export async function deleteReportPattern(id: string): Promise<void> {
  const res = await commands.deleteReportPattern(id);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to delete report pattern");
}

export async function generateReportData(jobId: string): Promise<ReportData> {
  const res = await commands.generateReportData(jobId);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to generate report data");
}
