import { describe, it, expect, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";

vi.mock("@/src/api/extension", () => ({
  listTags: vi.fn(),
}));

import { listTags } from "@/src/api/extension";
import { useExtractorTags } from "../use-extractor-tags";

const mockedListTags = vi.mocked(listTags);

function wrapper({ children }: { children: ReactNode }) {
  return (
    <SWRConfig value={{ dedupingInterval: 0, provider: () => new Map() }}>
      {children}
    </SWRConfig>
  );
}

const mockTags = [
  { name: "url", source: { kind: "builtin" }, label: "Site URL" },
  { name: "tag:og_image", source: { kind: "extractor", extractor_id: "e1", extractor_name: "OG" }, label: "OG Image" },
  { name: "score", source: { kind: "builtin" }, label: "SEO Score" },
];

describe("useExtractorTags", () => {
  it("splits tags into extractor and builtin groups", async () => {
    mockedListTags.mockResolvedValue(mockTags as never);

    const { result } = renderHook(() => useExtractorTags(), { wrapper });

    await waitFor(() => expect(result.current.tags.length).toBe(3));

    expect(result.current.extractorTags).toHaveLength(1);
    expect(result.current.extractorTags[0].name).toBe("tag:og_image");
    expect(result.current.builtinTags).toHaveLength(2);
  });

  it("returns empty arrays while loading", () => {
    mockedListTags.mockReturnValue(new Promise(() => {}));
    const { result } = renderHook(() => useExtractorTags(), { wrapper });

    expect(result.current.tags).toEqual([]);
    expect(result.current.extractorTags).toEqual([]);
    expect(result.current.builtinTags).toEqual([]);
    expect(result.current.isLoading).toBe(true);
  });
});
