import { ArrowLeft } from "lucide-react"
import { Button } from "../../ui/button"

export function BackButton({ onClick }: { onClick: () => void }) {
    return (
        <Button
            variant="ghost"
            size="icon"
            onClick={onClick}
            className="shrink-0"
        >
            <ArrowLeft className="h-4 w-4" />
        </Button>
    )
}
