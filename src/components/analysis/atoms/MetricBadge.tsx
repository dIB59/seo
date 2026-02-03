export function MetricBadge({ label, value }: { label: string; value: number | string }) {
    return (
        <div className="text-center p-2 rounded-lg bg-muted/50">
            <p className="text-lg font-semibold">{value}</p>
            <p className="text-[10px] text-muted-foreground">{label}</p>
        </div>
    )
}