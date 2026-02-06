import { Card, CardContent, CardHeader } from "@/src/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import { ImageIcon, Eye } from "lucide-react"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/src/components/ui/tooltip"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/src/components/ui/dialog"
import StatusBadge from "@/src/components/page-detail-view/atoms/StatusBadge"
import type { ImageElement } from "@/src/lib/types"
import { useState } from "react"
import { Badge } from "@/src/components/ui/badge"
import { Button } from "@/src/components/ui/button"

export default function ImagesTab({ images }: { images: ImageElement[] }) {
    if (!images || images.length === 0) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <ImageIcon className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No images found on this page</p>
                    <p className="text-sm text-muted-foreground mt-1">Backend needs to populate the images array</p>
                </CardContent>
            </Card>
        )
    }

    const withAlt = images.filter((img) => img.alt !== null && img.alt.length > 0).length
    const missingAlt = images.length - withAlt

    const [previewSrc, setPreviewSrc] = useState<string | null>(null)

    const isDataURI = (s: string) => s.startsWith("data:") || s.includes("base64,")
    const isLongSrc = (s: string) => s.length > 2000
    const shouldOfferPreview = (s: string) => isDataURI(s) || isLongSrc(s)
    const truncate = (s: string, n = 120) => (s.length > n ? `${s.slice(0, n)}...` : s)

    return (
        <>
            <Card>
                <CardHeader className="pb-3">
                    <div className="flex items-center gap-3">
                        <Badge variant="outline" className="bg-success/15 text-success border-success/20">{withAlt} with alt</Badge>
                        {missingAlt > 0 && (
                            <Badge variant="outline" className="bg-destructive/15 text-destructive border-destructive/20">{missingAlt} missing alt</Badge>
                        )}
                    </div>
                </CardHeader>
                <CardContent className="pt-0">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Source</TableHead>
                                <TableHead>Alt Text</TableHead>
                                <TableHead className="w-[100px] text-center">Status</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {images.map((image, idx) => (
                                <TableRow key={idx}>
                                    <TableCell className="max-w-[250px]">
                                        <div className="flex items-center gap-2">
                                            <TooltipProvider>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <span className="text-sm truncate block font-mono text-muted-foreground cursor-default">
                                                            {shouldOfferPreview(image.src) ? truncate(image.src, 120) : image.src}
                                                        </span>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p className="max-w-md break-all font-mono text-xs">{shouldOfferPreview(image.src) ? `${truncate(image.src, 600)} (truncated)` : image.src}</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>

                                            {shouldOfferPreview(image.src) && (
                                                <Button variant="ghost" size="sm" onClick={() => setPreviewSrc(image.src)} aria-label="Preview image">
                                                    <Eye className="h-3 w-3" />
                                                </Button>
                                            )}
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        {image.alt ? (
                                            <TooltipProvider>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <span className="text-sm truncate block max-w-[300px] cursor-default">{image.alt}</span>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p className="max-w-md break-words">{image.alt}</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>
                                        ) : (
                                            <span className="text-muted-foreground italic">Missing</span>
                                        )}
                                    </TableCell>
                                    <TableCell className="text-center">
                                        <StatusBadge hasContent={!!image.alt} label={image.alt ? "OK" : "Missing"} />
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>

            <Dialog open={!!previewSrc} onOpenChange={() => setPreviewSrc(null)}>
                <DialogContent className="max-w-3xl max-h-[90vh] overflow-auto">
                    <DialogHeader>
                        <DialogTitle>Image Preview</DialogTitle>
                    </DialogHeader>
                    <div className="p-4 flex justify-center">
                        {previewSrc && <img src={previewSrc} alt="preview" className="max-w-full max-h-[70vh] object-contain" />}
                    </div>
                </DialogContent>
            </Dialog>
        </>
    )
}
