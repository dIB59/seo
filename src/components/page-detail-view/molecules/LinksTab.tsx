import { Card, CardContent, CardHeader } from "@/src/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import { Link2, ExternalLink, Globe } from "lucide-react"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/src/components/ui/tooltip"
import { Badge } from "@/src/components/ui/badge"
import type { LinkElement } from "@/src/lib/types"
import { cn } from "@/src/lib/utils"

export default function LinksTab({ links }: { links: LinkElement[] }) {
    if (!links || links.length === 0) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <Link2 className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No links found on this page</p>
                    <p className="text-sm text-muted-foreground mt-1">Backend needs to populate the links array</p>
                </CardContent>
            </Card>
        )
    }

    console.log("LinksTab links:", links);
    const internalLinks = links.filter((l) => !l.is_external).length
    const externalLinks = links.length - internalLinks

    return (
        <Card>
            <CardHeader className="pb-3">
                <div className="flex items-center gap-3">
                    <Badge variant="outline" className="bg-primary/15 text-primary border-primary/20">
                        <Globe className="h-3 w-3 mr-1" />
                        {internalLinks} internal
                    </Badge>
                    <Badge variant="outline" className="bg-muted">
                        <ExternalLink className="h-3 w-3 mr-1" />
                        {externalLinks} external
                    </Badge>
                </div>
            </CardHeader>
            <CardContent className="pt-0">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>URL</TableHead>
                            <TableHead>Anchor Text</TableHead>
                            <TableHead className="w-[80px] text-center">Type</TableHead>
                            <TableHead className="w-[80px] text-center">Status</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {links.map((link, idx) => (
                            <TableRow key={idx}>
                                <TableCell className="max-w-[250px]">
                                    <TooltipProvider>
                                        <Tooltip>
                                            <TooltipTrigger asChild>
                                                <span className="text-sm truncate block font-mono text-muted-foreground cursor-default">{link.href}</span>
                                            </TooltipTrigger>
                                            <TooltipContent>
                                                <p className="max-w-md break-all font-mono text-xs">{link.href}</p>
                                            </TooltipContent>
                                        </Tooltip>
                                    </TooltipProvider>
                                </TableCell>
                                <TableCell>
                                    {link.text ? (
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <span className="text-sm truncate block max-w-[300px] cursor-default">{link.text}</span>
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p className="max-w-md break-words">{link.text}</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    ) : (
                                        <span className="text-muted-foreground italic">No text</span>
                                    )}
                                </TableCell>
                                <TableCell className="text-center">
                                    <Badge variant="outline" className={cn("text-xs", link.is_external ? "bg-muted": "bg-primary/15 text-primary border-primary/20")}>
                                        {link.is_external ? "Ext" : "Int"}
                                    </Badge>
                                </TableCell>
                                <TableCell className="text-center">
                                    {link.status_code ? (
                                        <Badge variant="outline" className={cn("text-xs font-mono", link.status_code >= 200 && link.status_code < 300 ? "bg-success/15 text-success border-success/20" : link.status_code >= 400 ? "bg-destructive/15 text-destructive border-destructive/20" : "bg-warning/15 text-warning border-warning/20")}>
                                            {link.status_code}
                                        </Badge>
                                    ) : (
                                        <span className="text-muted-foreground">-</span>
                                    )}
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}
