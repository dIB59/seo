import { Settings2 } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Badge } from "@/src/components/ui/badge"
import { CollapsibleTrigger } from "@/src/components/ui/collapsible"

interface SettingsTriggerProps {
    isModified: boolean
}

export function SettingsTrigger({ isModified }: SettingsTriggerProps) {
    return (
        <CollapsibleTrigger asChild>
            <Button variant="ghost" size="sm" className="text-muted-foreground hover:text-foreground transition-colors">
                <Settings2 className="h-4 w-4 mr-2" />
                Analysis Settings
                {isModified && (
                    <Badge variant="secondary" className="ml-2 px-1.5 py-0 h-4 min-w-4 text-[10px] bg-primary/10 text-primary border-none">
                        Modified
                    </Badge>
                )}
            </Button>
        </CollapsibleTrigger>
    )
}
