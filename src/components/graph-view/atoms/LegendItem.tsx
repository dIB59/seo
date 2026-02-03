export interface LegendItemProps { color: string; label: string }
export default function LegendItem({ color, label }: LegendItemProps) {
    return (
        <div className="flex items-center gap-2">
            <div className="w-3 h-3 rounded-full" style={{ backgroundColor: color }} />
            {label}
        </div>
    )
}
