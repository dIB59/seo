"use client";

import { useRef, useState } from "react";
import { Bot, Cpu, Loader2, Sparkles } from "lucide-react";
import { Button } from "@/src/components/ui/button";
import { generateAnalysis, type AiSource } from "@/src/api/ai";
import { useMutation } from "@/src/hooks/use-mutation";
import type { CompleteAnalysisResponse } from "@/src/api/analysis";

const SOURCE_LABEL: Record<AiSource, { label: string; icon: React.ReactNode }> = {
  gemini: { label: "Gemini", icon: <Sparkles className="h-3.5 w-3.5" /> },
  local:  { label: "Local Model", icon: <Cpu className="h-3.5 w-3.5" /> },
};

export function AiInsightsTab({ data }: { data: CompleteAnalysisResponse }) {
  const [insights, setInsights] = useState<{ text: string; source: AiSource } | null>(null);
  const topRef = useRef<HTMLDivElement>(null);

  const generate = useMutation(
    async () => {
      const res = await generateAnalysis(data);
      if (res.isErr()) throw new Error(res.unwrapErr());
      const result = res.unwrap();
      setInsights(result);
      topRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
      return result;
    },
  );

  return (
    <div className="space-y-4">
      <div ref={topRef} />
      {/* Action bar */}
      <div className="flex items-center gap-3 p-4 rounded-lg border border-border/50 bg-card/30">
        <div className="flex-1">
          <p className="text-sm font-medium">Generate AI Insights</p>
          <p className="text-xs text-muted-foreground mt-0.5">
            Uses Gemini if configured, otherwise the active local model.
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          disabled={generate.isLoading}
          onClick={() => generate.execute()}
          className="gap-2 h-8 text-xs shrink-0"
        >
          {generate.isLoading ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <Bot className="h-3.5 w-3.5" />
          )}
          {generate.isLoading ? "Generating…" : insights ? "Regenerate" : "Generate"}
        </Button>
      </div>

      {/* Error */}
      {generate.error && !generate.isLoading && (
        <div className="p-4 rounded-lg border border-destructive/30 bg-destructive/5 text-sm text-destructive">
          {generate.error}
        </div>
      )}

      {/* Result */}
      {insights && !generate.isLoading && (
        <div className="rounded-lg border border-border/50 bg-card/30 overflow-hidden">
          <div className="flex items-center gap-2 px-4 py-2.5 border-b border-border/40 bg-muted/20">
            {SOURCE_LABEL[insights.source].icon}
            <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
              {SOURCE_LABEL[insights.source].label}
            </span>
          </div>
          <div className="p-4">
            <p className="text-sm leading-relaxed whitespace-pre-wrap text-foreground/90">
              {insights.text}
            </p>
          </div>
        </div>
      )}

      {/* Empty state */}
      {!insights && !generate.isLoading && !generate.error && (
        <div className="flex flex-col items-center justify-center py-16 text-center gap-3 text-muted-foreground">
          <Bot className="h-10 w-10 opacity-20" />
          <p className="text-sm">
            Click Generate to analyse this result with AI.
          </p>
          <p className="text-xs opacity-60">
            Configure Gemini API key or a local model in Settings.
          </p>
        </div>
      )}
    </div>
  );
}
