import React from "react";
import type { ReportData } from "@/src/bindings";
import { SeoReportDocument } from "./SeoReport";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { toast } from "sonner";

// Register Geist for the PDF report. Fonts are served from /public/fonts
// so they're available to @react-pdf/renderer at runtime in the
// renderer process. Done lazily and once.
let fontsRegistered = false;
async function ensureFontsRegistered() {
  if (fontsRegistered) return;
  const { Font } = await import("@react-pdf/renderer");
  // Inter (sans) + JetBrains Mono. We use Inter rather than Geist
  // because Geist's GSUB ligatures (`fi`, `ff`, `tt`) collapse in
  // react-pdf, dropping characters from words like "trafic" /
  // "diference" / "paterns". Inter renders cleanly and is visually
  // close enough that the report still feels app-aligned.
  Font.register({
    family: "Inter",
    fonts: [
      { src: "/fonts/Inter-Regular.ttf", fontWeight: "normal" },
      { src: "/fonts/Inter-Regular.ttf", fontWeight: "normal", fontStyle: "italic" },
      { src: "/fonts/Inter-Medium.ttf",  fontWeight: "medium" },
      { src: "/fonts/Inter-Medium.ttf",  fontWeight: "medium",  fontStyle: "italic" },
      { src: "/fonts/Inter-Bold.ttf",    fontWeight: "bold" },
      { src: "/fonts/Inter-Bold.ttf",    fontWeight: "bold",    fontStyle: "italic" },
    ],
  });
  Font.register({ family: "Inter-Medium", src: "/fonts/Inter-Medium.ttf" });
  Font.register({ family: "Inter-Bold",   src: "/fonts/Inter-Bold.ttf" });
  Font.register({ family: "JetBrainsMono", src: "/fonts/JetBrainsMono-Regular.ttf" });
  Font.register({ family: "JetBrainsMono-Medium", src: "/fonts/JetBrainsMono-Medium.ttf" });
  // Disable hyphenation — Geist + soft hyphens look bad in narrow PDF columns.
  Font.registerHyphenationCallback((word) => [word]);
  fontsRegistered = true;
}

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
  await ensureFontsRegistered();

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
