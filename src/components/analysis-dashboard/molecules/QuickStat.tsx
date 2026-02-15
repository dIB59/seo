import { Clock, FileText, ImageIcon, Link2 } from "lucide-react"
import { Card, CardContent } from "../../ui/card"
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
	const totalMissingAlt = pages.reduce((acc, p) => acc + p.images_without_alt, 0)
	const totalInternalLinks = pages.reduce((acc, p) => acc + p.internal_links, 0)

	return (
		<Card className="bg-card/40 backdrop-blur-sm border-white/5 shadow-sm relative group">
			<div className="absolute inset-0 bg-gradient-to-br from-purple-500/5 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
			<CardContent className="p-4 relative z-10">
				<h3 className="text-xs font-medium uppercase tracking-wider text-muted-foreground mb-4">Quick Stats</h3>
				<div className="grid grid-cols-2 gap-4">
					<StatItem icon={Clock} label="Avg Load Time" value={`${summary.avg_load_time.toFixed(2)}s`} />
					<StatItem icon={FileText} label="Total Words" value={summary.total_words.toLocaleString()} />
					<StatItem
						icon={ImageIcon}
						label="Images"
						value={
							<span>
								{totalImages}
								{totalMissingAlt > 0 && (
									<span className="text-destructive/80 ml-1 font-sans">({totalMissingAlt})</span>
								)}
							</span>
						}
					/>
					<StatItem icon={Link2} label="Internal Links" value={totalInternalLinks} />
				</div>
			</CardContent>
		</Card>
	)
}
