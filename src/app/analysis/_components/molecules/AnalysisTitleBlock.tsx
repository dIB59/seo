import { ExternalLink } from "lucide-react";
import { open } from "@tauri-apps/plugin-shell";

export function AnalysisTitleBlock({ url }: { url: string }) {
  return (
    <div className="min-w-0">
      <div className="flex items-center gap-2">
        <h2 className="text-2xl font-bold tracking-tight truncate">{url}</h2>
        <button
          type="button"
          aria-label="Open URL in external browser"
          className="shrink-0 opacity-50 hover:opacity-100 transition-opacity"
          onClick={() => open(url)}
        >
          <ExternalLink className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}
