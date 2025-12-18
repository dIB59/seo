import { PageAnalysisData } from "@/src/lib/types"
import { Zap, Eye, Shield, Search } from "lucide-react"
import { ScoreRing } from "../atoms/ScoreRing"

export function LighthouseScores({ page }: { page: PageAnalysisData }) {
    if (!page.lighthouse_performance) return null

    const scores = [
        { label: "Performance", value: page.lighthouse_performance, icon: Zap },
        { label: "Accessibility", value: page.lighthouse_accessibility, icon: Eye },
        { label: "Best Practices", value: page.lighthouse_best_practices, icon: Shield },
        { label: "SEO", value: page.lighthouse_seo, icon: Search },
    ]

    return (
        <div className="grid grid-cols-4 gap-2">
            {scores.map((score) => (
                <div key={score.label} className="flex flex-col items-center gap-1 p-2 rounded-lg bg-muted/50">
                    <ScoreRing score={score.value || 0} size="sm" />
                    <span className="text-[10px] text-muted-foreground text-center">{score.label}</span>
                </div>
            ))}
        </div>
    )
}
