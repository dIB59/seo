"use client";

import { Suspense } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import { useAnalysis } from "@/src/hooks/use-analysis";
import { LoadingState, ErrorState } from "@/src/components/ui/page-states";
import { AnalysisDashboard } from "@/src/app/analysis/_components/AnalysisDashboard";

function AnalysisPageContent() {
  const searchParams = useSearchParams();
  const router = useRouter();

  const id = searchParams.get("id") ?? "";
  const { result, isLoading, isError } = useAnalysis(id);

  if (isLoading) {
    return <LoadingState message="Loading analysis..." />;
  }

  if (isError || !result) {
    return (
      <ErrorState
        title="Error Loading Analysis"
        description="Could not retrieve analysis data."
        backLabel="Back to Home"
        onBack={() => router.push("/")}
      />
    );
  }

  return (
    <main className="min-h-screen p-6 max-w-7xl mx-auto">
      <AnalysisDashboard
        data={result}
        onBack={() => router.push("/")}
        onSelectPage={(index: number) => router.push(`/analysis/details?id=${id}&index=${index}`)}
      />
    </main>
  );
}

export default function AnalysisPage() {
  return (
    <Suspense fallback={<LoadingState message="Loading analysis..." />}>
      <AnalysisPageContent />
    </Suspense>
  );
}
