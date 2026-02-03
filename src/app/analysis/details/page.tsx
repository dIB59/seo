"use client"

import { useSearchParams, useRouter } from "next/navigation"
import { useAnalysis } from "@/src/hooks/use-analysis"
import { Button } from "@/src/components/ui/button"
import { ArrowLeft, Loader2 } from "lucide-react"
import { PageDetailData } from "@/src/lib/types"
import { PageDetailView } from "@/src/components/page-detail-view/PageDetailView"

export default function PageDetailPage() {
    const searchParams = useSearchParams()
    const router = useRouter()

    // Fallback if params are missing
    const id = searchParams.get("id") ?? ""
    const indexStr = searchParams.get("index") ?? "0"
    const currentIndex = parseInt(indexStr, 10)

    const { result, isLoading, isError } = useAnalysis(id)

    if (isLoading) {
        return (
            <div className="flex flex-col items-center justify-center min-h-screen">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
                <p className="mt-4 text-muted-foreground">Loading page...</p>
            </div>
        )
    }

    if (isError || !result || isNaN(currentIndex) || !result.pages[currentIndex]) {
        return (
            <div className="flex flex-col items-center justify-center min-h-screen p-4 text-center">
                <h1 className="text-2xl font-bold text-destructive mb-2">Page Not Found</h1>
                <p className="text-muted-foreground mb-4">Could not load the requested page.</p>
                <Button onClick={() => router.push(`/analysis?id=${id}`)}>
                    <ArrowLeft className="mr-2 h-4 w-4" />
                    Back to Analysis
                </Button>
            </div>
        )
    }

    const pages = result.pages as PageDetailData[]
    const page = pages[currentIndex]

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
    )
}
