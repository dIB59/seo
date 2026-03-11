import type { Metadata } from "next";
import AnalysisPageClient from "./AnalysisPageClient";

export const metadata: Metadata = {
  title: "Analysis",
  description: "View crawl and SEO analysis results",
};

export default async function AnalysisPage({
  searchParams,
}: {
  searchParams: Promise<{ id?: string }>;
}) {
  const params = await searchParams;
  return <AnalysisPageClient id={params.id ?? ""} />;
}
