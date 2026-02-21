export default function LoadingOverlay() {
    return (
        <div className="absolute inset-0 flex items-center justify-center bg-background/60 backdrop-blur-sm z-10">
            <div className="flex flex-col items-center gap-3">
                <div className="relative">
                    <div className="w-10 h-10 border-4 border-primary/20 rounded-full animate-pulse" />
                    <div className="absolute inset-0 w-10 h-10 border-4 border-primary border-t-transparent rounded-full animate-spin" />
                </div>
                <div className="space-y-1 text-center">
                    <div className="text-sm font-medium text-foreground">Loading Graph</div>
                    <div className="text-xs text-muted-foreground">Rendering network topology…</div>
                </div>
            </div>
        </div>
    )
}
