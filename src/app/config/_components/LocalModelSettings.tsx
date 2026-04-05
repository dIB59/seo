"use client";

import { CheckCircle2, Download, HardDrive, Trash2, X } from "lucide-react";
import { Badge } from "@/src/components/ui/badge";
import { Button } from "@/src/components/ui/button";
import { Skeleton } from "@/src/components/ui/skeleton";
import { useLocalModels } from "@/src/hooks/use-local-models";
import type { ModelInfo } from "@/src/bindings";
import type { DownloadProgress } from "@/src/hooks/use-local-models";

// ── Tier styling ──────────────────────────────────────────────────────────────

const TIER_STYLES: Record<string, { label: string; className: string }> = {
  large:  { label: "Large",  className: "bg-purple-500/15 text-purple-400 border-purple-500/30" },
  medium: { label: "Medium", className: "bg-blue-500/15 text-blue-400 border-blue-500/30" },
  small:  { label: "Small",  className: "bg-green-500/15 text-green-400 border-green-500/30" },
};

function TierBadge({ tier }: { tier: string }) {
  const style = TIER_STYLES[tier] ?? { label: tier, className: "" };
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border ${style.className}`}>
      {style.label}
    </span>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(0)} MB`;
  return `${(bytes / 1e3).toFixed(0)} KB`;
}

// ── Download progress bar ─────────────────────────────────────────────────────

function ProgressBar({ progress }: { progress: number }) {
  const pct = progress < 0 ? null : Math.round(progress * 100);
  return (
    <div className="w-full space-y-1">
      <div className="h-1.5 w-full rounded-full bg-muted overflow-hidden">
        {pct !== null ? (
          <div
            className="h-full bg-primary rounded-full transition-all duration-300"
            style={{ width: `${pct}%` }}
          />
        ) : (
          // Indeterminate shimmer for unknown total
          <div className="h-full w-1/3 bg-primary rounded-full animate-[shimmer_1.5s_infinite]" />
        )}
      </div>
      <p className="text-xs text-muted-foreground">
        {pct !== null ? `${pct}% downloaded` : "Downloading…"}
      </p>
    </div>
  );
}

// ── Individual model card ─────────────────────────────────────────────────────

function ModelCard({
  model,
  progress,
  onDownload,
  onCancel,
  onDelete,
  onActivate,
}: {
  model: ModelInfo;
  progress: DownloadProgress | undefined;
  onDownload: () => void;
  onCancel: () => void;
  onDelete: () => void;
  onActivate: () => void;
}) {
  const isDownloading = progress !== undefined;

  return (
    <div
      className={`p-4 rounded-lg border transition-all duration-200 ${
        model.is_active
          ? "border-primary/50 bg-primary/5 ring-1 ring-primary/20"
          : "border-border/50 bg-card/30 hover:border-border/80"
      }`}
    >
      {/* Header row */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-medium text-sm leading-tight">{model.name}</span>
            <TierBadge tier={model.tier} />
            {model.is_active && (
              <span className="inline-flex items-center gap-1 text-xs text-primary font-medium">
                <CheckCircle2 className="h-3 w-3" />
                Active
              </span>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-0.5 leading-snug">{model.description}</p>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          <span className="text-xs text-muted-foreground font-mono">
            {formatBytes(model.size_bytes)}
          </span>
        </div>
      </div>

      {/* Download progress */}
      {isDownloading && (
        <div className="mt-3">
          <ProgressBar progress={progress.progress} />
          {progress.totalBytes > 0 && (
            <p className="text-xs text-muted-foreground mt-0.5">
              {formatBytes(progress.downloadedBytes)} / {formatBytes(progress.totalBytes)}
            </p>
          )}
        </div>
      )}

      {/* Actions */}
      <div className="mt-3 flex items-center gap-2">
        {isDownloading ? (
          <Button variant="outline" size="sm" onClick={onCancel} className="h-7 text-xs gap-1.5">
            <X className="h-3 w-3" />
            Cancel
          </Button>
        ) : model.is_downloaded ? (
          <>
            {!model.is_active && (
              <Button variant="outline" size="sm" onClick={onActivate} className="h-7 text-xs gap-1.5">
                <CheckCircle2 className="h-3 w-3" />
                Set Active
              </Button>
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={onDelete}
              className="h-7 text-xs gap-1.5 text-destructive hover:text-destructive hover:bg-destructive/10"
            >
              <Trash2 className="h-3 w-3" />
              Delete
            </Button>
          </>
        ) : (
          <Button variant="outline" size="sm" onClick={onDownload} className="h-7 text-xs gap-1.5">
            <Download className="h-3 w-3" />
            Download
          </Button>
        )}
      </div>
    </div>
  );
}

// ── Section header ────────────────────────────────────────────────────────────

function SectionHeader() {
  return (
    <div className="flex items-start gap-3 p-4 rounded-lg border border-border/50 bg-card/30">
      <HardDrive className="h-5 w-5 text-primary mt-0.5 shrink-0" />
      <div>
        <p className="text-sm font-medium">On-device AI inference</p>
        <p className="text-xs text-muted-foreground leading-relaxed mt-0.5">
          Download a model to generate SEO insights without sending data to external APIs.
          Models are stored locally and run entirely on your machine.
        </p>
      </div>
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export function LocalModelSettings() {
  const { models, isLoading, downloading, download, cancel, remove, activate } = useLocalModels();

  if (isLoading) {
    return (
      <div className="space-y-3 animate-in fade-in duration-300">
        <Skeleton className="h-[72px] w-full rounded-lg" />
        {[0, 1, 2].map((i) => (
          <Skeleton key={i} className="h-[110px] w-full rounded-lg" />
        ))}
      </div>
    );
  }

  const downloaded = models.filter((m) => m.is_downloaded);

  return (
    <div className="space-y-4 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <SectionHeader />

      {downloaded.length > 0 && (
        <p className="text-xs text-muted-foreground px-1">
          {downloaded.length} model{downloaded.length !== 1 ? "s" : ""} on disk
          {models.find((m) => m.is_active)
            ? ` · Active: ${models.find((m) => m.is_active)!.name}`
            : " · No active model"}
        </p>
      )}

      <div className="space-y-3">
        {models.map((model) => (
          <ModelCard
            key={model.id}
            model={model}
            progress={downloading[model.id]}
            onDownload={() => download(model.id)}
            onCancel={() => cancel(model.id)}
            onDelete={() => remove(model.id)}
            onActivate={() => activate(model.id)}
          />
        ))}
      </div>
    </div>
  );
}
