export function DiscoveryProgress({ count }: { count: number | null }) {
    return (
        <div className="mt-2 flex items-center gap-2">
            <div className="flex-1 overflow-hidden h-1.5 bg-muted rounded-full">
                <div className="h-full bg-blue-500 animate-[shimmer_2s_infinite] w-[40%]" />
            </div>
            <span className="text-xs text-muted-foreground whitespace-nowrap">
                Discovered {count ?? 0} pages...
            </span>
        </div>
    )
}
