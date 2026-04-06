import type { CompleteAnalysisResponse } from "@/src/api/analysis";
import { generateReport, generateCSV } from "@/src/lib/report-generator";
import { logger } from "@/src/lib/logger";
import { generateReportData } from "@/src/api/report";
import { generateReportPdf } from "@/src/lib/pdf/generate";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile, writeFile } from "@tauri-apps/plugin-fs";
import { toast } from "sonner";

function getScoreColor(score: number): [number, number, number] {
  if (score >= 80) return [34, 197, 94]; // green
  if (score >= 50) return [234, 179, 8]; // yellow
  return [239, 68, 68]; // red
}

function formatDomain(url: string): string {
  return url.replace(/^https?:\/\//, "").replace(/[^a-z0-9]/gi, "-");
}

function formatDate(date: Date = new Date()): string {
  return date.toISOString().split("T")[0];
}

async function saveFile(
  content: string | Uint8Array<ArrayBuffer>,
  defaultFilename: string,
  filters?: { name: string; extensions: string[] }[],
): Promise<void> {
  try {
    const filePath = await save({
      defaultPath: defaultFilename,
      filters: filters || [],
    });

    if (filePath) {
      if (typeof content === "string") {
        await writeTextFile(filePath, content);
      } else {
        // For binary data (like PDFs), use writeFile
        await writeFile(filePath, content);
      }
      toast.success("File saved successfully");
      logger.log("File saved successfully:", filePath);
    } else {
      toast.info("File save was cancelled");
      logger.log("Save cancelled by user");
    }
  } catch (error: unknown) {
    if (isCancellationError(error)) {
      toast.info("Save cancelled");
      logger.log("Save cancelled by user (caught error)");
      return;
    }

    logger.error("Error saving file:", error);
    toast.error("Failed to save file. Using browser download instead.");
    // Fallback to browser download if Tauri API fails
    fallbackDownload(content, defaultFilename);
  }
}

function isCancellationError(error: unknown): boolean {
  const msg = String(error);
  const json = JSON.stringify(error, Object.getOwnPropertyNames(error));
  return (
    msg.includes("cancelled") ||
    msg.includes("-999") ||
    msg.includes("NSURLErrorDomain") ||
    msg.includes("Operation couldn't be completed") ||
    json.includes("-999") ||
    (typeof error === "object" && error !== null && "code" in error && (error as { code: unknown }).code === -999)
  );
}

function fallbackDownload(content: string | Uint8Array<ArrayBuffer>, filename: string) {
  // Determine MIME type based on extension
  const extension = filename.split(".").pop()?.toLowerCase();
  let mimeType = "text/plain";

  if (extension === "pdf") {
    mimeType = "application/pdf";
  } else if (extension === "csv") {
    mimeType = "text/csv";
  } else if (extension === "txt") {
    mimeType = "text/plain";
  }

  const blob = new Blob([content], { type: mimeType });

  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);

  toast.success("File downloaded successfully");
  logger.log("File downloaded successfully (fallback):", filename);
}


export async function generatePDF(result: CompleteAnalysisResponse): Promise<void> {
  try {
    const reportData = await generateReportData(result.analysis.id);
    await generateReportPdf(reportData);
  } catch (err) {
    logger.error("PDF generation failed:", err);
    toast.error("Failed to generate PDF report");
  }
}

export async function downloadTextReport(result: CompleteAnalysisResponse): Promise<void> {
  const reportText = generateReport(result);
  const filename = `seo-report-${formatDomain(result.analysis.url)}-${formatDate()}.txt`;

  await saveFile(reportText, filename, [
    { name: "Text Files", extensions: ["txt"] },
    { name: "All Files", extensions: ["*"] },
  ]);
}

export async function downloadCSVReport(result: CompleteAnalysisResponse): Promise<void> {
  const csvData = generateCSV(result);
  const filename = `seo-data-${formatDomain(result.analysis.url)}-${formatDate()}.csv`;

  await saveFile(csvData, filename, [
    { name: "CSV Files", extensions: ["csv"] },
    { name: "All Files", extensions: ["*"] },
  ]);
}
