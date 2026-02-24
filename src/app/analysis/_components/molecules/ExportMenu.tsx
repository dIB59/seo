
import {
    Download,
    ChevronDown,
    FileText,
    TableIcon,
} from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/src/components/ui/dropdown-menu"

export function ExportMenu({
    onPDF,
    onText,
    onCSV,
}: {
    onPDF: () => void
    onText: () => void
    onCSV: () => void
}) {
    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button variant="outline" className="shrink-0 bg-background/50 hover:bg-primary/10 hover:text-primary hover:border-primary/20 transition-all duration-300 shadow-sm">
                    <Download className="h-4 w-4 mr-2" />
                    Export Report
                    <ChevronDown className="h-4 w-4 ml-2 opacity-50" />
                </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={onPDF}>
                    <FileText className="h-4 w-4 mr-2" />
                    Download PDF
                </DropdownMenuItem>

                <DropdownMenuItem onClick={onText}>
                    <FileText className="h-4 w-4 mr-2" />
                    Download Text Report
                </DropdownMenuItem>

                <DropdownMenuItem onClick={onCSV}>
                    <TableIcon className="h-4 w-4 mr-2" />
                    Download CSV Data
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    )
}
