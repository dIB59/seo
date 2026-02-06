import { ScoreRing } from "@/src/components/analysis-dashboard/atoms/ScoreRing"
import type { PageDetailData } from "@/src/lib/types"

export default function ScoreOverview({ page }: { page: PageDetailData }) {
    const scores = [
        { label: "Performance", value: page.lighthouse_performance, color: "text-orange-500" },
        { label: "Accessibility", value: page.lighthouse_accessibility, color: "text-blue-500" },
        { label: "Best Practices", value: page.lighthouse_best_practices, color: "text-purple-500" },
        { label: "SEO", value: page.lighthouse_seo, color: "text-green-500" },
    ]

    return (
        <div className="grid grid-cols-4 gap-4">
            {scores.map((score) => (
                <div key={score.label} className="flex flex-col items-center gap-2 p-4 rounded-lg bg-muted/50">
                    <ScoreRing score={Number(score.value?.toPrecision(3)) || 0} size="md" />
                    <div className="flex items-center gap-1.5">
                        <span className="text-sm font-medium">{score.label}</span>
                    </div>
                </div>
            ))}
        </div>
    )
}
