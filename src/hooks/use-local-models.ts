import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ModelDownloadEvent, ModelInfo } from "@/src/bindings";
import {
  cancelModelDownload,
  deleteLocalModel,
  downloadLocalModel,
  listLocalModels,
  setActiveLocalModel,
} from "@/src/api/local-model";
import { toast } from "sonner";

export type DownloadProgress = {
  downloadedBytes: number;
  totalBytes: number;
  /** 0–1, or -1 if total unknown */
  progress: number;
};

export function useLocalModels() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [downloading, setDownloading] = useState<Record<string, DownloadProgress>>({});
  // Track which models are in the "starting download" state (command sent, no event yet)
  const pendingDownloads = useRef<Set<string>>(new Set());

  const refresh = useCallback(async () => {
    const res = await listLocalModels();
    if (res.isOk()) setModels(res.unwrap());
  }, []);

  // Initial load
  useEffect(() => {
    setIsLoading(true);
    refresh().finally(() => setIsLoading(false));
  }, [refresh]);

  // Listen for download progress events from backend
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setup = async () => {
      unlisten = await listen<ModelDownloadEvent>("model-download-event", (e) => {
        const { modelId, status, downloadedBytes, totalBytes, progress } = e.payload;

        if (status === "downloading") {
          setDownloading((prev) => ({
            ...prev,
            [modelId]: { downloadedBytes, totalBytes, progress },
          }));
        } else {
          // Terminal state — remove from in-progress map and refresh the model list
          setDownloading((prev) => {
            const next = { ...prev };
            delete next[modelId];
            return next;
          });
          pendingDownloads.current.delete(modelId);

          if (status === "completed") {
            toast.success("Model downloaded", {
              description: `Ready to activate.`,
            });
            refresh();
          } else if (status === "failed") {
            toast.error("Download failed", { description: `Could not download model ${modelId}.` });
          }
          // "cancelled" is intentional — no toast needed
        }
      });
    };

    setup();
    return () => { unlisten?.(); };
  }, [refresh]);

  const download = useCallback(async (modelId: string) => {
    pendingDownloads.current.add(modelId);
    // Optimistically show a 0% progress so the UI reacts immediately
    setDownloading((prev) => ({
      ...prev,
      [modelId]: { downloadedBytes: 0, totalBytes: 0, progress: -1 },
    }));

    const res = await downloadLocalModel(modelId);
    if (res.isErr()) {
      pendingDownloads.current.delete(modelId);
      setDownloading((prev) => {
        const next = { ...prev };
        delete next[modelId];
        return next;
      });
      toast.error("Download failed", { description: res.unwrapErr() });
    }
  }, []);

  const cancel = useCallback(async (modelId: string) => {
    await cancelModelDownload(modelId);
    // Event listener will handle cleanup when "cancelled" arrives
  }, []);

  const remove = useCallback(async (modelId: string) => {
    const res = await deleteLocalModel(modelId);
    if (res.isOk()) {
      toast.success("Model deleted");
      refresh();
    } else {
      toast.error("Failed to delete model", { description: res.unwrapErr() });
    }
  }, [refresh]);

  const activate = useCallback(async (modelId: string) => {
    const res = await setActiveLocalModel(modelId);
    if (res.isOk()) {
      toast.success("Active model updated");
      refresh();
    } else {
      toast.error("Failed to set active model", { description: res.unwrapErr() });
    }
  }, [refresh]);

  return { models, isLoading, downloading, download, cancel, remove, activate };
}
