import { Label } from "@/src/components/ui/label"
import { Switch } from "@/src/components/ui/switch"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/src/components/ui/tooltip"
import { HelpCircle } from "lucide-react"

interface SettingToggleProps {
    id: string
    label: string
    description?: string
    tooltip?: string
    checked: boolean
    onCheckedChange: (checked: boolean) => void
}

export function SettingToggle({ id, label, description, tooltip, checked, onCheckedChange }: SettingToggleProps) {
    return (
        <div className="flex items-center justify-between space-x-2">
            <div className="flex flex-col gap-0.5">
                <div className="flex items-center gap-1.5">
                    <Label htmlFor={id} className="text-sm font-medium cursor-pointer">
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
                {description && <span className="text-[11px] text-muted-foreground leading-tight">{description}</span>}
            </div>
            <Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
        </div>
    )
}
