import { Card, CardContent } from "@/src/components/ui/card"
import { FileText, Clock, ImageIcon, Link2, ExternalLink } from "lucide-react"
import { Badge } from "@/src/components/ui/badge"
import PageStructure from "@/src/components/page-detail-view/molecules/PageStructure"

export default function PageInfoCard({ page }: { page: any }) {
    return (
        <Card>
            <CardContent className="py-4">
                <div className="flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                        <h2 className="text-lg font-semibold truncate">{page.url}</h2>
                        <a
                            href={page.url}
                            rel="noopener noreferrer"
                            className="shrink-0"
                            target="_blank"
                            onClick={(e) => {
                                e.preventDefault()
                                open(page.url)
                            }}
                        >
                            <ExternalLink className="h-4 w-4 text-muted-foreground hover:text-foreground" />
                        </a>
                    </div>
                    <p className="text-sm text-muted-foreground truncate">{page.title || "No title"}</p>
                    <div className="flex flex-wrap gap-4 text-sm">
                        <div className="flex items-center gap-1.5">
                            <Badge variant="outline" className="font-mono">{page.status_code || "N/A"}</Badge>
                            <span className="text-muted-foreground">Status</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <Clock className="h-4 w-4 text-muted-foreground" />
                            <span className={page.load_time ? (page.load_time < 1 ? "text-success" : page.load_time < 2 ? "text-warning" : "text-destructive") : "text-muted-foreground"}>
                                {page.load_time.toFixed(2)}s
                            </span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <FileText className="h-4 w-4 text-muted-foreground" />
                            <span>{page.word_count.toLocaleString()} words</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <ImageIcon className="h-4 w-4 text-muted-foreground" />
                            <span>
                                {page.image_count} images
                                {page.images_without_alt > 0 && (
                                    <span className="text-destructive ml-1">({page.images_without_alt} no alt)</span>
                                )}
                            </span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <Link2 className="h-4 w-4 text-muted-foreground" />
                            <span>{page.internal_links} int / {page.external_links} ext</span>
                        </div>

                        {/* Page structure badges (mobile/structured-data/missing alt) */}
                        <div className="w-full mt-2">
                            <PageStructure page={page} />
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}
