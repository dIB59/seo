import { RotateCcw } from "lucide-react"
import { Button } from "@/src/components/ui/button"

interface SettingsResetButtonProps {
    onReset: () => void
}

export function SettingsResetButton({ onReset }: SettingsResetButtonProps) {
    return (
        <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
                e.stopPropagation()
                onReset()
            }}
            className="text-[11px] h-8 text-muted-foreground hover:text-destructive transition-colors"
        >
            <RotateCcw className="h-3 w-3 mr-1.5" />
            Reset to Defaults
        </Button>
    )
}
