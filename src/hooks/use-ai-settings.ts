import useSWR from "swr";
import {
  get_gemini_api_key,
  get_gemini_persona,
  get_gemini_enabled,
  get_gemini_prompt_blocks,
} from "@/src/api/ai";
import type { PromptBlock } from "@/src/lib/types";

async function fetchAiSettings() {
  const [keyRes, personaRes, enabledRes, blocksRes] = await Promise.all([
    get_gemini_api_key(),
    get_gemini_persona(),
    get_gemini_enabled(),
    get_gemini_prompt_blocks(),
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
