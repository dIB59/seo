"use client";

import { Info } from "lucide-react";
import { Label } from "@/src/components/ui/label";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/src/components/ui/tooltip";

interface PersonaSettingsProps {
    persona: string;
    setPersona: (persona: string) => void;
}

export function PersonaSettings({ persona, setPersona }: PersonaSettingsProps) {
    return (
        <div className="space-y-4 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="space-y-2 p-4 border border-border/50 rounded-lg bg-card/30 transition-all duration-300 focus-within:border-primary/40 focus-within:ring-1 focus-within:ring-primary/10 hover:border-border/80 relative group">
                {/* Glow effect - subtler */}
                <div className="absolute inset-0 bg-primary/2 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none rounded-lg" />

                <div className="flex items-center gap-2 relative">
                    <Label>System Persona</Label>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Info className="h-3.5 w-3.5 text-muted-foreground/70 hover:text-primary transition-colors cursor-help" />
                        </TooltipTrigger>
                        <TooltipContent>
                            <p className="max-w-xs">Defines the AI&apos;s role and tone. Use this to customize the analysis perspective (e.g., &apos;Strict Technical Auditor&apos; or &apos;Marketing Specialist&apos;).</p>
                        </TooltipContent>
                    </Tooltip>
                </div>
                <textarea
                    className="flex min-h-[200px] w-full rounded-md border border-input/50 bg-background/50 px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus:bg-background disabled:cursor-not-allowed disabled:opacity-50 font-mono transition-all duration-200 resize-y focus-visible:border-primary relative z-10"
                    value={persona}
                    onChange={(e) => setPersona(e.target.value)}
                    placeholder="You are an expert SEO auditor..."
                />
                <div className="flex justify-between items-center relative z-10">
                    <p className="text-xs text-muted-foreground">
                        Defines the personality and expertise level of the AI.
                    </p>
                    <span className="text-[10px] text-muted-foreground font-mono bg-muted/50 px-2 py-0.5 rounded">Markdown Supported</span>
                </div>
            </div>
        </div>
    );
}
