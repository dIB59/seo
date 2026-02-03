export default function LoadingOverlay() {
    return (
        <div className="absolute inset-0 flex items-center justify-center bg-muted/10 z-10">
            <div className="flex flex-col items-center gap-3">
                <div className="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin" />
                <div className="text-sm text-muted-foreground">Loading Graph...</div>
            </div>
        </div>
    )
}
