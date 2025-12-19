import { ExternalLink } from "lucide-react"

export function AnalysisTitleBlock({
    url,
    pageCount,
    completedAt,
}: {
    url: string
    pageCount: number
    completedAt?: string | null
}) {
    return (
        <div className="min-w-0">
            <div className="flex items-center gap-2">
                <h2 className="text-xl font-semibold truncate">
                    {url}
                </h2>
                <a
                    href={url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="shrink-0"
                >
                    <ExternalLink className="h-4 w-4 text-muted-foreground hover:text-foreground" />
                </a>
            </div>
            <p className="text-sm text-muted-foreground">
                {pageCount} pages analyzed ·{" "}
                {completedAt ? new Date(completedAt).toLocaleDateString() : ""}
            </p>
        </div>
    )
}
