import { CompleteAnalysisResponse } from "@/src/lib/types"
import {
    downloadCSVReport,
    downloadTextReport,
    generatePDF,
} from "@/src/lib/export-utils"
import { AnalysisTitleBlock } from "../molecules/AnalysisTitleBlock"
import { ExportMenu } from "../molecules/ExportMenu"
import { BackButton } from "../atoms/BackButton"
import { Calendar, Layers } from "lucide-react"
import { format } from "date-fns"

export function AnalysisHeader({
    onBack,
    result,
}: {
    onBack: () => void
    result: CompleteAnalysisResponse
}) {
    const { analysis, pages } = result

    return (
        <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-6 animate-in fade-in slide-in-from-top-4 duration-500">
            <div className="flex items-start gap-4 min-w-0 flex-1">
                <BackButton onClick={onBack} />

                <div className="space-y-1 min-w-0">
                    <AnalysisTitleBlock
                        url={analysis.url}
                    />

                    <div className="flex items-center gap-4 text-xs text-muted-foreground font-mono mt-2">
                        <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-muted/30 border border-border/30">
                            <Calendar className="h-3 w-3" />
                            <span>{analysis.completed_at ? format(new Date(analysis.completed_at), "MMM d, HH:mm") : "Just now"}</span>
                        </div>
                        <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-muted/30 border border-border/30">
                            <Layers className="h-3 w-3" />
                            <span>{pages.length} Pages Scanned</span>
                        </div>
                        <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-muted/30 border border-border/30">
                            <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                            <span>Analysis Complete</span>
                        </div>
                    </div>
                </div>
            </div>

            <div className="flex items-center gap-3 bg-card/20 border border-white/5 p-1.5 rounded-xl backdrop-blur-sm shadow-sm self-start xl:self-center">
                <span className="text-xs font-medium text-muted-foreground px-2">Actions</span>
                <div className="h-4 w-px bg-border/40" />
                <ExportMenu
                    onPDF={() => generatePDF(result)}
                    onText={() => downloadTextReport(result)}
                    onCSV={() => downloadCSVReport(result)}
                />
            </div>
        </div>
    )
}
