import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";
import { Result } from "@/src/lib/result";

function wrapper({ children }: { children: ReactNode }) {
  return <SWRConfig value={{ provider: () => new Map(), dedupingInterval: 0 }}>{children}</SWRConfig>;
}

vi.mock("@/src/api/ai", () => ({
  getApiKey: vi.fn(),
  getPersona: vi.fn(),
  getAiEnabled: vi.fn(),
  getPromptBlocks: vi.fn(),
}));

import {
  getApiKey,
  getPersona,
  getAiEnabled,
  getPromptBlocks,
} from "@/src/api/ai";
import { useAiSettings } from "@/src/hooks/use-ai-settings";

describe("useAiSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("aggregates Ok responses into a settings object", async () => {
    (getApiKey as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Ok("k"));
    (getPersona as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Ok("p"));
    (getAiEnabled as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Ok(true));
    (getPromptBlocks as ReturnType<typeof vi.fn>).mockResolvedValue(
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
    (getApiKey as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));
    (getPersona as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));
    (getAiEnabled as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));
    (getPromptBlocks as ReturnType<typeof vi.fn>).mockResolvedValue(Result.Err("x"));

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
