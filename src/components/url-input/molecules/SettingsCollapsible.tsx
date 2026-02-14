import type { ReactNode } from "react"
import { Settings2, RotateCcw } from "lucide-react"
import { Button } from "@/src/components/ui/button"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/src/components/ui/collapsible"
import { Badge } from "@/src/components/ui/badge"

interface SettingsCollapsibleProps {
    isOpen: boolean
    onOpenChange: (open: boolean) => void
    isModified: boolean
    onReset: () => void
    children: ReactNode
}

export function SettingsCollapsible({ isOpen, onOpenChange, isModified, onReset, children }: SettingsCollapsibleProps) {
    return (
        <Collapsible open={isOpen} onOpenChange={onOpenChange}>
            <div className="flex items-center justify-between">
                <CollapsibleTrigger asChild>
                    <Button variant="ghost" size="sm" className="text-muted-foreground hover:text-foreground transition-colors">
                        <Settings2 className="h-4 w-4 mr-2" />
                        Advanced Settings
                        {isModified && (
                            <Badge variant="secondary" className="ml-2 px-1.5 py-0 h-4 min-w-4 text-[10px] bg-primary/10 text-primary border-none">
                                Modified
                            </Badge>
                        )}
                    </Button>
                </CollapsibleTrigger>

                {isModified && (
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
                )}
            </div>
            <CollapsibleContent className="mt-4">
                <div className="p-4 bg-secondary/30 rounded-lg border border-border/50 backdrop-blur-sm">
                    {children}
                </div>
            </CollapsibleContent>
        </Collapsible>
    )
}
