export function StatItem({
    icon: Icon,
    label,
    value,
    subValue,
}: {
    icon: React.ComponentType<{ className?: string }>
    label: string
    value: React.ReactNode
    subValue?: string
}) {
    return (
        <div className="flex items-center gap-2 p-2 rounded-lg bg-muted/50">
            <Icon className="h-4 w-4 text-muted-foreground shrink-0" />
            <div className="min-w-0 flex-1">
                <p className="text-sm font-semibold truncate">{value}</p>
                <p className="text-[10px] text-muted-foreground truncate">{label}</p>
                {subValue && <p className="text-[10px] text-muted-foreground">{subValue}</p>}
            </div>
        </div>
    )
}

export function StatItemError({
    icon: Icon,
    label,
    value,
    subValue,
}: {
    icon: React.ComponentType<{ className?: string }>
    label: string
    value: React.ReactNode
    subValue?: string
}) {
    return (
        <div
            className="flex items-center gap-2 p-2 rounded-lg bg-card border-l-4"
            style={{ borderLeftColor: "red" }}
        >
            <Icon className="h-4 w-4 text-muted-foreground shrink-0" />
            <div className="min-w-0 flex-1">
                {/* value is the only red touch – uses the raw variable */}
                <p
                    className="text-sm font-semibold truncate"
                    style={{ color: 'hsl(var(--destructive))' }}
                >
                    {value}
                </p>

                {/* label & sub-value stay on the neutral scale */}
                <p className="text-[10px] text-muted-foreground truncate">{label}</p>
                {subValue && (
                    <p className="text-[10px] text-muted-foreground">{subValue}</p>
                )}
            </div>
        </div>
    )
}
