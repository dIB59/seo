import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";
import { Result } from "@/src/lib/result";

function wrapper({ children }: { children: ReactNode }) {
  return <SWRConfig value={{ provider: () => new Map(), dedupingInterval: 0 }}>{children}</SWRConfig>;
}

vi.mock("@/src/api/ai", () => ({
  get_gemini_api_key: vi.fn(),
  get_gemini_persona: vi.fn(),
  get_gemini_enabled: vi.fn(),
  get_gemini_prompt_blocks: vi.fn(),
}));

import {
  get_gemini_api_key,
  get_gemini_persona,
  get_gemini_enabled,
  get_gemini_prompt_blocks,
} from "@/src/api/ai";
import { useAiSettings } from "@/src/hooks/use-ai-settings";

describe("useAiSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("aggregates Ok responses into a settings object", async () => {
    (get_gemini_api_key as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Ok("k"));
    (get_gemini_persona as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Ok("p"));
    (get_gemini_enabled as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Ok(true));
    (get_gemini_prompt_blocks as ReturnType<typeof vi.fn>).mockResolvedValue(
      Result.Ok(JSON.stringify([{ id: "a", title: "t", content: "c" }])),
    );

    const { result } = renderHook(() => useAiSettings(), { wrapper });

    await waitFor(() => expect(result.current.settings).toBeDefined());
    expect(result.current.settings).toEqual({
      apiKey: "k",
      persona: "p",
      aiEnabled: true,
      blocks: [{ id: "a", title: "t", content: "c" }],
    });
  });

  it("falls back to safe defaults when calls Err", async () => {
    (get_gemini_api_key as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));
    (get_gemini_persona as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));
    (get_gemini_enabled as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));
    (get_gemini_prompt_blocks as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));

    const { result } = renderHook(() => useAiSettings(), { wrapper });

    await waitFor(() => expect(result.current.settings).toBeDefined());
    expect(result.current.settings).toEqual({
      apiKey: "",
      persona: "",
      aiEnabled: true,
      blocks: [],
    });
  });
});
