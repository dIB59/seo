import { ArrowLeft } from "lucide-react"
import { Button } from "../../ui/button"

export function BackButton({ onClick }: { onClick: () => void }) {
    return (
        <Button
            variant="ghost"
            size="sm"
            onClick={onClick}
            className="gap-2 pl-0 text-muted-foreground hover:text-foreground hover:bg-transparent group transition-colors -ml-2"
        >
            <div className="p-1 rounded-full bg-muted/40 group-hover:bg-primary/10 group-hover:text-primary transition-colors">
                <ArrowLeft className="h-4 w-4" />
            </div>
            <span className="font-medium">Back to Dashboard</span>
        </Button>
    )
}
