import { CompleteAnalysisResult } from "@/src/lib/types"
import {
    downloadCSVReport,
    downloadTextReport,
    generatePDF,
} from "@/src/lib/export-utils"
import { AnalysisTitleBlock } from "../molecules/AnalysisTitleBlock"
import { ExportMenu } from "../molecules/ExportMenu"
import { BackButton } from "../atoms/BackButton"

export function AnalysisHeader({
    onBack,
    result,
}: {
    onBack: () => void
    result: CompleteAnalysisResult
}) {
    const { analysis, pages } = result

    return (
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div className="flex items-center gap-3 min-w-0">
                <BackButton onClick={onBack} />

                <AnalysisTitleBlock
                    url={analysis.url}
                    pageCount={pages.length}
                    completedAt={analysis.completed_at}
                />
            </div>

            <ExportMenu
                onPDF={() => generatePDF(result)}
                onText={() => downloadTextReport(result)}
                onCSV={() => downloadCSVReport(result)}
            />
        </div>
    )
}
