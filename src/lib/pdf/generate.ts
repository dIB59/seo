import React from "react";
import type { ReportData } from "@/src/bindings";
import { SeoReportDocument } from "./SeoReport";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { toast } from "sonner";

function formatDomain(url: string): string {
  return url.replace(/^https?:\/\//, "").replace(/[^a-z0-9]/gi, "-");
}

function formatDate(): string {
  return new Date().toISOString().split("T")[0];
}

/**
 * Generate a PDF from ReportData and prompt the user to save it.
 * Uses @react-pdf/renderer so it runs entirely in the renderer process.
 */
export async function generateReportPdf(data: ReportData): Promise<void> {
  const { pdf } = await import("@react-pdf/renderer");

  toast.info("Generating PDF report…");

  try {
    const blob = await pdf(React.createElement(SeoReportDocument, { data })).toBlob();
    const arrayBuffer = await blob.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);

    const filename = `seo-report-${formatDomain(data.url)}-${formatDate()}.pdf`;

    const filePath = await save({
      defaultPath: filename,
      filters: [
        { name: "PDF Files", extensions: ["pdf"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });

    if (!filePath) {
      toast.info("Save cancelled");
      return;
    }

    await writeFile(filePath, bytes);
    toast.success("PDF report saved");
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    if (msg.includes("cancel") || msg.includes("-999")) {
      toast.info("Save cancelled");
      return;
    }
    console.error("[PDF]", err);
    toast.error("Failed to generate PDF");
  }
}
