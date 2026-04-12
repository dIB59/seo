import useSWR from "swr";
import {
  getApiKey,
  getPersona,
  getAiEnabled,
  getPromptBlocks,
} from "@/src/api/ai";
import type { PromptBlock } from "@/src/lib/types";

async function fetchAiSettings() {
  const [keyRes, personaRes, enabledRes, blocksRes] = await Promise.all([
    getApiKey(),
    getPersona(),
    getAiEnabled(),
    getPromptBlocks(),
  ]);

  let blocks: PromptBlock[] = [];
  if (blocksRes.isOk()) {
    const saved = blocksRes.unwrap();
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed)) blocks = parsed;
      } catch {
        console.error("useAiSettings: Failed to parse blocks");
      }
    }
  }

  return {
    apiKey: keyRes.isOk() ? keyRes.unwrap() || "" : "",
    persona: personaRes.isOk() ? personaRes.unwrap() : "",
    aiEnabled: enabledRes.isOk() ? enabledRes.unwrap() : true,
    blocks,
  };
}

export function useAiSettings() {
  const { data, isLoading, mutate, error } = useSWR("ai-settings", fetchAiSettings, {
    revalidateOnFocus: false,
    revalidateOnReconnect: false,
  });

  return {
    settings: data,
    isLoading,
    isError: error,
    mutate,
  };
}
