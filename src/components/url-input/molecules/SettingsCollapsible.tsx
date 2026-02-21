import type { ReactNode } from "react"
import { Collapsible, CollapsibleContent } from "@/src/components/ui/collapsible"
import { SettingsTrigger } from "../atoms/SettingsTrigger"
import { SettingsResetButton } from "../atoms/SettingsResetButton"

interface SettingsCollapsibleProps {
    isOpen: boolean
    onOpenChange: (open: boolean) => void
    isModified: boolean
    onReset: () => void
    children: ReactNode
}

export function AnalysisSettingsCollapsible({ isOpen, onOpenChange, isModified, onReset, children }: SettingsCollapsibleProps) {
    return (
        <Collapsible open={isOpen} onOpenChange={onOpenChange}>
            <div className="flex items-center justify-between">
                <SettingsTrigger isModified={isModified} />

                {isModified && (
                    <SettingsResetButton onReset={onReset} />
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
