import type { Metadata } from "next";
import PageDetailPageClient from "./PageDetailPageClient";

export const metadata: Metadata = {
  title: "Page Details",
  description: "Inspect page-level SEO details and issues",
};

export default async function PageDetailPage({
  searchParams,
}: {
  searchParams: Promise<{ id?: string; index?: string }>;
}) {
  const params = await searchParams;
  return <PageDetailPageClient id={params.id ?? ""} index={params.index ?? "0"} />;
}
