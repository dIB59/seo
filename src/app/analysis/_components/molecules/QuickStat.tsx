import { Clock, FileText, ImageIcon, Link2 } from "lucide-react"
import { Card, CardContent } from "@/src/components/ui/card"
import { StatItem } from "../atoms/Stat"
import { CompleteAnalysisResponse } from "@/src/lib/types"

export function QuickStatsCard({
	summary,
	pages,
}: {
	summary: CompleteAnalysisResponse["summary"]
	pages: CompleteAnalysisResponse["pages"]
}) {
	const totalImages = pages.reduce((acc, p) => acc + p.image_count, 0)
	const totalInternalLinks = pages.reduce((acc, p) => acc + p.internal_links, 0)
	const avgLoadTime = pages.reduce((acc, p) => acc + p.load_time, 0) / pages.length

	return (
		<Card className="bg-card/40 backdrop-blur-sm border-white/5 shadow-sm relative group overflow-hidden">
			<div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
			<CardContent className="p-4 relative z-10">
				<h3 className="text-xs font-medium uppercase tracking-wider text-muted-foreground mb-4">Quick Stats</h3>
				<div className="grid grid-cols-2 gap-4">
					<StatItem icon={Clock} label="Avg Load Time" value={avgLoadTime.toFixed(2)} />
					<StatItem icon={FileText} label="Total Words" value={summary.total_words} />
					<StatItem
						icon={ImageIcon}
						label="Images"
						value={totalImages}

					/>
					<StatItem icon={Link2} label="Internal Links" value={totalInternalLinks} />
				</div>
			</CardContent>
		</Card>
	)
}
