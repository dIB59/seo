import { PageAnalysisData } from "@/src/lib/types";
import { cn } from "@/src/lib/utils";
import { Dialog, DialogContent, DialogTitle } from "@radix-ui/react-dialog";
import { Separator } from "@radix-ui/react-dropdown-menu";
import { BarChart3, Clock, FileCode, Link2, FileText, Smartphone, ImageIcon } from "lucide-react";
import { DialogHeader } from "../../ui/dialog";
import { MetricBadge } from "../atoms/MetricBadge";
import { StatItemError, StatItem } from "../atoms/Stat";
import { LighthouseDetailedView } from "../molecules/LighthouseDetailedView";
import { Badge } from "../../ui/badge";

const isBroken = (p: PageAnalysisData) => p.status_code! >= 400 || p.status_code! < 200;

export function PageDetailModal({
    page,
    open,
    onClose,
}: { page: PageAnalysisData | null; open: boolean; onClose: () => void }) {
    if (!page) return null

    return isBroken(page) ? (
        <BrokenPageModal page={page} open={open} onClose={onClose} />
    ) : (
        <HealthyPageModal page={page} open={open} onClose={onClose} />
    )
}

function BrokenPageModal({ page, open, onClose }: { page: PageAnalysisData; open: boolean; onClose: () => void }) {
    return (
        <Dialog open={open} onOpenChange={onClose}>
            <DialogContent className="max-w-2xl max-h-[90vh] overflow-hidden grid">
                <DialogHeader className="bg-destructive/5 rounded-t-lg -mx-6 -mt-6 px-6 py-4 truncate">
                    <DialogTitle className="truncate pr-8 text-destructive">{page.url}</DialogTitle>
                    <p className="text-sm text-muted-foreground truncate">{page.title || "No title"}</p>
                </DialogHeader>

                {/* scrollable body */}
                <div className="overflow-auto">
                    <h4 className="text-sm font-medium mb-3">Content Metrics</h4>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                        <StatItemError icon={BarChart3} label="Status" value={page.status_code} />
                        <StatItemError icon={Clock} label="Load Time" value={`${page.load_time.toFixed(2)}s`} />
                        <StatItemError icon={FileCode} label="Content Size" value={`${(page.content_size / 1024).toFixed(1)}KB`} />
                        <StatItemError icon={Link2} label="Links" value={`${page.internal_links}/${page.external_links}`} />
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    )
}


function HealthyPageModal({
    page,
    open,
    onClose,
}: { page: PageAnalysisData | null; open: boolean; onClose: () => void }) {
    if (!page) return null

    return (
        <Dialog open={open} onOpenChange={onClose}>
            <DialogContent className="max-w-3xl max-h-[90vh] overflow-auto">
                <DialogHeader className="truncate">
                    <DialogTitle className="truncate pr-8">{page.url}</DialogTitle>
                    <p className="text-sm text-muted-foreground truncate">{page.title || "No title"}</p>
                </DialogHeader>

                <div className="space-y-6">
                    <LighthouseDetailedView page={page} />

                    <Separator />

                    <div>
                        <h4 className="text-sm font-medium mb-3">Meta Information</h4>
                        <div className="space-y-2 text-sm">
                            {[
                                { label: "Title", value: page.title },
                                { label: "Description", value: page.meta_description },
                                { label: "Keywords", value: page.meta_keywords },
                                { label: "Canonical", value: page.canonical_url },
                            ].map(({ label, value }) => (
                                <div key={label} className="grid grid-cols-[100px_1fr] gap-2">
                                    <span className="text-muted-foreground">{label}:</span>
                                    <span className="truncate">{value || <span className="text-muted-foreground">None</span>}</span>
                                </div>
                            ))}
                        </div>
                    </div>

                    <Separator />

                    <div>
                        <h4 className="text-sm font-medium mb-3">Content Metrics</h4>
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                            <StatItem icon={FileText} label="Word Count" value={page.word_count.toLocaleString()} />
                            <StatItem icon={Clock} label="Load Time" value={`${page.load_time.toFixed(2)}s`} />
                            <StatItem icon={FileCode} label="Content Size" value={`${(page.content_size / 1024).toFixed(1)}KB`} />
                            <StatItem icon={BarChart3} label="Status Code" value={page.status_code || "N/A"} />
                        </div>
                    </div>

                    <Separator />

                    <div>
                        <h4 className="text-sm font-medium mb-3">Page Structure</h4>
                        <div className="grid grid-cols-3 md:grid-cols-6 gap-3">
                            <MetricBadge label="H1 Tags" value={page.h1_count} />
                            <MetricBadge label="H2 Tags" value={page.h2_count} />
                            <MetricBadge label="H3 Tags" value={page.h3_count} />
                            <MetricBadge label="Images" value={page.image_count} />
                            <MetricBadge label="Int. Links" value={page.internal_links} />
                            <MetricBadge label="Ext. Links" value={page.external_links} />
                        </div>
                    </div>

                    <Separator />

                    <div className="flex flex-wrap gap-2">
                        <Badge
                            variant="outline"
                            className={cn(
                                "text-xs",
                                page.mobile_friendly ? "bg-success/15 text-success border-success/20" : "bg-muted",
                            )}
                        >
                            <Smartphone className="h-3 w-3 mr-1" />
                            {page.mobile_friendly ? "Mobile Friendly" : "Not Mobile Friendly"}
                        </Badge>
                        <Badge
                            variant="outline"
                            className={cn(
                                "text-xs",
                                page.has_structured_data ? "bg-success/15 text-success border-success/20" : "bg-muted",
                            )}
                        >
                            <FileCode className="h-3 w-3 mr-1" />
                            {page.has_structured_data ? "Has Structured Data" : "No Structured Data"}
                        </Badge>
                        {page.images_without_alt > 0 && (
                            <Badge variant="outline" className="text-xs bg-destructive/15 text-destructive border-destructive/20">
                                <ImageIcon className="h-3 w-3 mr-1" />
                                {page.images_without_alt} Images Missing Alt
                            </Badge>
                        )}
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    )
}