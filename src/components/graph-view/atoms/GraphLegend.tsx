import type { LegendItemProps } from "./LegendItem"
import LegendItem from "./LegendItem"

export default function GraphLegend() {
    return (
        <div className="p-4 border-t flex gap-4 text-xs text-muted-foreground justify-center">
            <LegendItem color="#46c773ff" label="Healthy" />
            <LegendItem color="#e8aa3fff" label="Warning" />
            <LegendItem color="#f14444ff" label="Critical" />
            <LegendItem color="#ff0000ff" label="Error" />
        </div>
    )
}
