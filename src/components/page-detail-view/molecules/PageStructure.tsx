import { Badge } from "@/src/components/ui/badge"
import { FileCode, Smartphone, ImageIcon } from "lucide-react"
import { cn } from "@/src/lib/utils"
import type { PageDetailData } from "@/src/lib/types"

export default function PageStructure({ page }: { page: PageDetailData }) {
    return (
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
    )
}
