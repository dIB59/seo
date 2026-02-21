import { Label } from "@/src/components/ui/label"
import { Input } from "@/src/components/ui/input"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/src/components/ui/tooltip"
import { HelpCircle } from "lucide-react"

interface SettingInputProps {
    id: string
    label: string
    tooltip?: string
    value: number
    onChange: (value: number) => void
    min?: number
    max?: number
    disabled?: boolean
}

export function SettingInput({ id, label, tooltip, value, onChange, min, max, disabled }: SettingInputProps) {
    return (
        <div className={`space-y-2 ${disabled ? "opacity-60 cursor-not-allowed" : ""}`}>
            <div className="flex items-center gap-1.5">
                <Label htmlFor={id} className="text-sm font-medium">
                    {label}
                </Label>
                {tooltip && (
                    <TooltipProvider>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <HelpCircle className="h-3.5 w-3.5 text-muted-foreground cursor-help" />
                            </TooltipTrigger>
                            <TooltipContent>
                                <p className="max-w-xs text-xs">{tooltip}</p>
                            </TooltipContent>
                        </Tooltip>
                    </TooltipProvider>
                )}
            </div>
            <Input
                id={id}
                type="number"
                value={value}
                onChange={(e) => onChange(Number.parseInt(e.target.value) || 0)}
                className="bg-background h-9 text-sm"
                min={min}
                max={max}
                disabled={disabled}
            />
        </div>
    )
}
