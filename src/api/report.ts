import { commands } from "@/src/bindings";
import type {
  ReportPattern,
  ReportPatternParams,
  ReportData,
  ReportTemplate,
} from "@/src/bindings";

export type { ReportPattern, ReportPatternParams, ReportData, ReportTemplate };

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

// ── Report Templates ────────────────────────────────────────────────────────

export async function listReportTemplates(): Promise<ReportTemplate[]> {
  const res = await commands.listReportTemplates();
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to list report templates");
}

export async function getReportTemplate(id: string): Promise<ReportTemplate> {
  const res = await commands.getReportTemplate(id);
  if (res.status === "ok") return res.data;
  throw new Error(res.error ?? "Failed to get report template");
}

export async function createReportTemplate(template: ReportTemplate): Promise<void> {
  const res = await commands.createReportTemplate(template);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to create report template");
}

export async function updateReportTemplate(template: ReportTemplate): Promise<void> {
  const res = await commands.updateReportTemplate(template);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to update report template");
}

export async function setActiveReportTemplate(id: string): Promise<void> {
  const res = await commands.setActiveReportTemplate(id);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to set active template");
}

export async function deleteReportTemplate(id: string): Promise<void> {
  const res = await commands.deleteReportTemplate(id);
  if (res.status !== "ok") throw new Error(res.error ?? "Failed to delete report template");
}
