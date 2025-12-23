import { X } from "lucide-react"
import { Button } from "@/src/components/ui/button"

export function CancelButton({ onClick }: { onClick: () => void }) {
    return (
        <Button
            variant="ghost"
            size="sm"
            onClick={onClick}
            className="text-destructive hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
        >
            <X className="h-4 w-4" />
        </Button>
    )
}
