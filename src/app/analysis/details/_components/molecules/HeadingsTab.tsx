import { Card, CardContent } from "@/src/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/src/components/ui/tooltip"
import { Heading as HeadingIcon } from "lucide-react"
import type { HeadingElement } from "@/src/lib/types"

export default function HeadingsTab({ headings }: { headings: HeadingElement[] }) {
    if (!headings || headings.length === 0) {
        return (
            <Card>
                <CardContent className="py-12 text-center">
                    <HeadingIcon className="h-12 w-12 text-muted-foreground mx-auto mb-3" />
                    <p className="text-muted-foreground">No headings found on this page</p>
                    <p className="text-sm text-muted-foreground mt-1">Backend needs to populate the headings array</p>
                </CardContent>
            </Card>
        )
    }

    const tagColors: Record<string, string> = {
        h1: "bg-primary text-primary-foreground",
        h2: "bg-primary/80 text-primary-foreground",
        h3: "bg-primary/60 text-primary-foreground",
        h4: "bg-primary/40 text-primary-foreground",
        h5: "bg-primary/30",
        h6: "bg-primary/20",
    }

    return (
        <Card>
            <CardContent className="pt-6">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[80px]">Tag</TableHead>
                            <TableHead>Content</TableHead>
                            <TableHead className="w-[100px] text-right">Length</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {headings.map((heading, idx) => (
                            <TableRow key={idx}>
                                <TableCell>
                                    <span className={`font-mono uppercase ${tagColors[heading.tag]}`}>{heading.tag}</span>
                                </TableCell>
                                <TableCell className="max-w-[400px]">
                                    {heading.text ? (
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <span className="text-sm truncate block cursor-default">{heading.text}</span>
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p className="max-w-md break-words">{heading.text}</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    ) : (
                                        <span className="text-muted-foreground italic">Empty</span>
                                    )}
                                </TableCell>
                                <TableCell className="text-right">{heading.text.length}</TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}
