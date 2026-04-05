"use client";

import { useCallback, useEffect, useState } from "react";
import { Cpu, Info, Sparkles } from "lucide-react";
import { Save } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/src/components/ui/button";
import { Input } from "@/src/components/ui/input";
import { Label } from "@/src/components/ui/label";
import { Separator } from "@/src/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/src/components/ui/tooltip";
import { commands } from "@/src/bindings";
import { set_gemini_api_key } from "@/src/api/ai";
import { LocalModelSettings } from "./LocalModelSettings";
import type { AiSource } from "@/src/api/ai";

// ── Source picker ─────────────────────────────────────────────────────────────

function SourceOption({
  id,
  active,
  icon,
  title,
  description,
  onClick,
}: {
  id: AiSource;
  active: boolean;
  icon: React.ReactNode;
  title: string;
  description: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-start gap-3 p-4 rounded-lg border text-left transition-all duration-200 ${
        active
          ? "border-primary/50 bg-primary/5 ring-1 ring-primary/20"
          : "border-border/50 bg-card/30 hover:border-border/80"
      }`}
    >
      <div
        className={`mt-0.5 p-1.5 rounded-md shrink-0 ${active ? "bg-primary/15 text-primary" : "bg-muted text-muted-foreground"}`}
      >
        {icon}
      </div>
      <div>
        <p className="text-sm font-medium leading-tight">{title}</p>
        <p className="text-xs text-muted-foreground mt-0.5 leading-snug">{description}</p>
      </div>
      <div className="ml-auto mt-1 shrink-0">
        <div
          className={`h-4 w-4 rounded-full border-2 transition-colors ${
            active ? "border-primary bg-primary" : "border-muted-foreground/40"
          }`}
        />
      </div>
    </button>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export function AiSettings() {
  const [source, setSourceState] = useState<AiSource>("gemini");
  const [apiKey, setApiKey] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // Load current settings
  useEffect(() => {
    Promise.all([commands.getAiSource(), commands.getGeminiApiKey()]).then(
      ([sourceRes, keyRes]) => {
        if (sourceRes.status === "ok") {
          setSourceState(sourceRes.data === "local" ? "local" : "gemini");
        }
        if (keyRes.status === "ok") setApiKey(keyRes.data ?? "");
        setIsLoading(false);
      },
    );
  }, []);

  const selectSource = useCallback(
    async (next: AiSource) => {
      if (next === source) return;
      setSourceState(next);
      const res = await commands.setAiSource(next);
      if (res.status === "error") {
        toast.error("Failed to save AI source");
        setSourceState(source); // rollback
      }
    },
    [source],
  );

  const saveGeminiKey = useCallback(async () => {
    setIsSaving(true);
    const res = await set_gemini_api_key(apiKey);
    setIsSaving(false);
    if (res.isOk()) {
      toast.success("API key saved");
    } else {
      toast.error("Failed to save API key");
    }
  }, [apiKey]);

  if (isLoading) return null;

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      {/* Source picker */}
      <div className="space-y-2">
        <Label className="text-sm font-medium">AI Source</Label>
        <p className="text-xs text-muted-foreground">
          Choose which AI generates insights for reports and the analysis dashboard.
        </p>
        <div className="space-y-2 mt-3">
          <SourceOption
            id="gemini"
            active={source === "gemini"}
            icon={<Sparkles className="h-4 w-4" />}
            title="Gemini (Cloud)"
            description="Google's Gemini API. Requires an API key. Best quality."
            onClick={() => selectSource("gemini")}
          />
          <SourceOption
            id="local"
            active={source === "local"}
            icon={<Cpu className="h-4 w-4" />}
            title="Local Model (On-device)"
            description="Runs entirely on your machine. No API key needed. Private by default."
            onClick={() => selectSource("local")}
          />
        </div>
      </div>

      <Separator className="bg-border/40" />

      {/* Source-specific config */}
      {source === "gemini" && (
        <div className="space-y-4">
          <div className="space-y-2 p-4 border border-border/50 rounded-lg bg-card/30 focus-within:border-primary/40 focus-within:ring-1 focus-within:ring-primary/10 hover:border-border/80 transition-all duration-300 relative overflow-hidden group">
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-primary/5 to-transparent translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000 pointer-events-none" />
            <div className="flex items-center gap-2">
              <Label>Gemini API Key</Label>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="h-3.5 w-3.5 text-muted-foreground/70 hover:text-primary transition-colors cursor-help" />
                </TooltipTrigger>
                <TooltipContent>
                  <p className="max-w-xs">Used for generating insights and analysis reports.</p>
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
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span>Required for Gemini analysis.</span>
              <a
                href="https://aistudio.google.com/"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-primary underline"
              >
                Get API Key
              </a>
            </div>
          </div>
          <Button onClick={saveGeminiKey} disabled={isSaving} size="sm" className="gap-2">
            <Save className="h-3.5 w-3.5" />
            Save Key
          </Button>
        </div>
      )}

      {source === "local" && <LocalModelSettings />}
    </div>
  );
}
