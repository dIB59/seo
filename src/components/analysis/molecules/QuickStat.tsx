import { CompleteAnalysisResult, PageAnalysisData } from "@/src/lib/types"
import { Clock, FileText, ImageIcon, Link2 } from "lucide-react"
import { Card, CardContent } from "../../ui/card"
import { StatItem } from "../atoms/Stat"

export function QuickStatsCard({
	summary,
	pages,
}: {
	summary: CompleteAnalysisResult["summary"]
	pages: PageAnalysisData[]
}) {
	const totalImages = pages.reduce((acc, p) => acc + p.image_count, 0)
	const totalMissingAlt = pages.reduce((acc, p) => acc + p.images_without_alt, 0)
	const totalInternalLinks = pages.reduce((acc, p) => acc + p.internal_links, 0)

	return (
		<Card>
			<CardContent className="p-6">
				<h3 className="text-sm font-semibold mb-3">Quick Stats</h3>
				<div className="grid grid-cols-2 gap-3">
					<StatItem icon={Clock} label="Avg Load Time" value={`${summary.avg_load_time.toFixed(2)}s`} />
					<StatItem icon={FileText} label="Total Words" value={summary.total_words.toLocaleString()} />
					<StatItem
						icon={ImageIcon}
						label="Images"
						value={
							<span>
								{totalImages}
								{totalMissingAlt > 0 && (
									<span className="text-destructive/80 text-xs ml-1">({totalMissingAlt} no alt)</span>
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
