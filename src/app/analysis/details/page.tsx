"use client";

import { useSearchParams, useRouter } from "next/navigation";
import { useAnalysis } from "@/src/hooks/use-analysis";
import { LoadingState, ErrorState } from "@/src/components/ui/page-states";
import type { PageDetailData, PageAnalysisData } from "@/src/lib/types";
import { PageDetailView } from "@/src/app/analysis/details/_components/PageDetailView";

export default function PageDetailPage() {
  const searchParams = useSearchParams();
  const router = useRouter();

  const id = searchParams.get("id") ?? "";
  const indexStr = searchParams.get("index") ?? "0";
  const currentIndex = parseInt(indexStr, 10);

  const { result, isLoading, isError } = useAnalysis(id);

  if (isLoading) {
    return <LoadingState message="Loading page..." />;
  }

  if (isError || !result || isNaN(currentIndex) || !result.pages[currentIndex]) {
    return (
      <ErrorState
        title="Page Not Found"
        description="Could not load the requested page."
        backLabel="Back to Analysis"
        onBack={() => router.push(`/analysis?id=${id}`)}
      />
    );
  }

  const pages = (result.pages as PageAnalysisData[]).map((p) => ({ ...p }) as PageDetailData);
  const page = pages[currentIndex];

  return (
    <div className="container max-w-5xl mx-auto py-6 px-4">
      <PageDetailView
        page={page}
        pages={pages}
        currentIndex={currentIndex}
        onBack={() => router.push(`/analysis?id=${id}`)}
        onNavigate={(newIndex) => router.push(`/analysis/details?id=${id}&index=${newIndex}`)}
      />
    </div>
  );
}
