"use client"

import { useSearchParams, useRouter } from "next/navigation"
import { AnalysisResults } from "@/src/components/analysis-results"
import { useAnalysis } from "@/src/hooks/use-analysis"
import { Button } from "@/src/components/ui/button"
import { ArrowLeft, Loader2 } from "lucide-react"

export default function AnalysisPage() {
    const searchParams = useSearchParams()
    const router = useRouter()

    // Fallback to "0" or handle null appropriately if you prefer
    const id = searchParams.get("id") ?? ""

    // useAnalysis handles string | number parsing
    const { result, isLoading, isError } = useAnalysis(id)

    if (isLoading) {
        return (
            <div className="flex flex-col items-center justify-center min-h-screen">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
                <p className="mt-4 text-muted-foreground">Loading analysis...</p>
            </div>
        )
    }

    if (isError || !result) {
        return (
            <div className="flex flex-col items-center justify-center min-h-screen p-4 text-center">
                <h1 className="text-2xl font-bold text-destructive mb-2">Error Loading Analysis</h1>
                <p className="text-muted-foreground mb-4">Could not retrieve analysis data.</p>
                <Button onClick={() => router.push("/")}>
                    <ArrowLeft className="mr-2 h-4 w-4" />
                    Back to Home
                </Button>
            </div>
        )
    }

    return (
        <main className="min-h-screen p-6 max-w-7xl mx-auto">
            <AnalysisResults
                result={result}
                onBack={() => router.push("/")}
                onSelectPage={(index) => router.push(`/analysis/details?id=${id}&index=${index}`)}
            />
        </main>
    )
}
