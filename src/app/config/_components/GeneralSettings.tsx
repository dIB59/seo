"use client";

import { Info } from "lucide-react";
import { Switch } from "@/src/components/ui/switch";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/src/components/ui/tooltip";

interface GeneralSettingsProps {
    apiKey: string;
    setApiKey: (key: string) => void;
    aiEnabled: boolean;
    setAiEnabled: (enabled: boolean) => void;
}

export function GeneralSettings({ apiKey, setApiKey, aiEnabled, setAiEnabled }: GeneralSettingsProps) {
    return (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
            {/* AI Toggle */}
            <div className="flex items-center justify-between p-4 border border-border/50 rounded-lg bg-card/30 transition-all duration-300 hover:border-border/80 group">
                <div className="space-y-0.5">
                    <Label className="text-base">Enable AI Features</Label>
                    <p className="text-sm text-muted-foreground">
                        Enable Gemini AI for specialized SEO insights.
                    </p>
                </div>
                <Switch checked={aiEnabled} onCheckedChange={setAiEnabled} />
            </div>

            {/* API Key */}
            <div className="space-y-2 p-4 border border-border/50 rounded-lg bg-card/30 transition-all duration-300 focus-within:border-primary/40 focus-within:ring-1 focus-within:ring-primary/10 hover:border-border/80 relative overflow-hidden group">
                <div className="absolute inset-0 bg-gradient-to-r from-transparent via-primary/5 to-transparent translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000 pointer-events-none" />
                <div className="flex items-center gap-2">
                    <Label>Gemini API Key</Label>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Info className="h-3.5 w-3.5 text-muted-foreground/70 hover:text-primary transition-colors cursor-help" />
                        </TooltipTrigger>
                        <TooltipContent>
                            <p className="max-w-xs">Values are encrypted at rest. Used for generating insights and analysis.</p>
                        </TooltipContent>
                    </Tooltip>
                </div>
                <Input
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    placeholder="AIza..."
                    className="font-mono bg-background/50 focus:bg-background transition-colors border-input/50 focus-visible:ring-0 focus-visible:border-primary"
                />
                <p className="text-xs text-muted-foreground flex justify-between">
                    <span>Required for analysis.</span>
                    <a href="https://aistudio.google.com/" target="_blank" rel="noopener noreferrer" className="hover:text-primary underline">Get API Key</a>
                </p>
            </div>
        </div>
    );
}
