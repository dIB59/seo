import LegendItem from "./LegendItem"

export default function GraphLegend() {
    return (
        <div className="absolute bottom-4 left-4 z-10 flex gap-3 px-3 py-2 text-xs text-muted-foreground bg-background/80 backdrop-blur border border-border/60 rounded-lg shadow-sm">
            <LegendItem color="#46c773ff" label="Healthy" />
            <LegendItem color="#e8aa3fff" label="Warning" />
            <LegendItem color="#f14444ff" label="Critical" />
            <LegendItem color="#ff0000ff" label="Error" />
        </div>
    )
}
