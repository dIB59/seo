import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/src/components/ui/table"
import { Badge } from "@/src/components/ui/badge"
import { CheckCircle2, XCircle } from "lucide-react"
import type { LighthouseSeoAudits } from "@/src/lib/types"

export default function SeoAuditBreakdown({ audits }: { audits: LighthouseSeoAudits }) {
    const auditItems = [
        { key: "document_title", label: "Document Title", audit: audits.document_title },
        { key: "meta_description", label: "Meta Description", audit: audits.meta_description },
        { key: "viewport", label: "Viewport Meta Tag", audit: audits.viewport },
        { key: "canonical", label: "Canonical URL", audit: audits.canonical },
        { key: "hreflang", label: "Hreflang Tags", audit: audits.hreflang },
        { key: "robots_txt", label: "Robots.txt Valid", audit: audits.robots_txt },
        { key: "crawlable_anchors", label: "Crawlable Anchors", audit: audits.crawlable_anchors },
        { key: "link_text", label: "Descriptive Link Text", audit: audits.link_text },
        { key: "image_alt", label: "Image Alt Attributes", audit: audits.image_alt },
        { key: "http_status_code", label: "HTTP Status Code", audit: audits.http_status_code },
        { key: "is_crawlable", label: "Page is Crawlable", audit: audits.is_crawlable },
    ]

    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>Audit</TableHead>
                    <TableHead className="w-[100px] text-right">Status</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {auditItems.map(({ key, label, audit }) => (
                    <TableRow key={key}>
                        <TableCell className="font-medium">
                            <div className="flex items-center gap-2">
                                {audit.passed ? <CheckCircle2 className="h-4 w-4 text-success" /> : <XCircle className="h-4 w-4 text-destructive" />}
                                {label}
                            </div>
                        </TableCell>
                        <TableCell className="text-right">
                            <Badge variant="outline" className={`text-xs ${audit.passed ? "bg-success/15 text-success border-success/20" : "bg-destructive/15 text-destructive border-destructive/20"}`}>
                                {audit.passed ? "Passed" : "Failed"}
                            </Badge>
                        </TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    )
}
