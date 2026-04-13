import { describe, it, expect, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";

vi.mock("@/src/api/analysis", () => ({
  getResult: vi.fn(),
}));

import { getResult } from "@/src/api/analysis";
import { useAnalysis } from "../use-analysis";

const mockedGetResult = vi.mocked(getResult);

function wrapper({ children }: { children: ReactNode }) {
  return (
    <SWRConfig value={{ dedupingInterval: 0, provider: () => new Map() }}>
      {children}
    </SWRConfig>
  );
}

const mockResult = {
  analysis: { id: "job-1", url: "https://example.com", status: "completed" },
  pages: [],
  issues: [],
  summary: { seo_score: 85, avg_load_time: 1.2, total_words: 5000, total_issues: 3 },
};

describe("useAnalysis", () => {
  it("returns loading state initially", () => {
    mockedGetResult.mockReturnValue(new Promise(() => {})); // never resolves
    const { result } = renderHook(() => useAnalysis("job-1"), { wrapper });
    expect(result.current.isLoading).toBe(true);
    expect(result.current.result).toBeUndefined();
  });

  it("returns result data on success", async () => {
    mockedGetResult.mockResolvedValue({
      isOk: () => true,
      unwrap: () => mockResult,
    } as never);

    const { result } = renderHook(() => useAnalysis("job-1"), { wrapper });

    await waitFor(() => expect(result.current.result).toBeDefined());
    expect(result.current.result?.analysis.url).toBe("https://example.com");
    expect(result.current.isLoading).toBe(false);
  });

  it("does not fetch when id is empty", () => {
    vi.clearAllMocks();
    const { result } = renderHook(() => useAnalysis(""), { wrapper });
    expect(result.current.isLoading).toBe(false);
    expect(result.current.result).toBeUndefined();
    expect(mockedGetResult).not.toHaveBeenCalled();
  });

  it("sets error on API failure", async () => {
    mockedGetResult.mockResolvedValue({
      isOk: () => false,
      unwrap: () => { throw new Error("Not found"); },
    } as never);

    const { result } = renderHook(() => useAnalysis("bad-id"), { wrapper });

    await waitFor(() => expect(result.current.isError).toBeDefined());
  });
});
