import { ExternalLink } from "lucide-react"
import { Button } from "@/src/components/ui/button"

export function ViewResultButton({ onClick }: { onClick: () => void }) {
    return (
        <Button
            variant="ghost"
            size="sm"
            onClick={onClick}
            className="text-primary hover:text-primary"
        >
            <ExternalLink className="h-4 w-4 mr-1" />
            View Results
        </Button>
    )
}
