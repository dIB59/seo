import { describe, it, expect, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";

vi.mock("@/src/api/extension", () => ({
  listTags: vi.fn(),
}));

import { listTags } from "@/src/api/extension";
import { useCheckFieldTags } from "../use-check-field-tags";

const mockedListTags = vi.mocked(listTags);

function wrapper({ children }: { children: ReactNode }) {
  return (
    <SWRConfig value={{ dedupingInterval: 0, provider: () => new Map() }}>
      {children}
    </SWRConfig>
  );
}

describe("useCheckFieldTags", () => {
  it("fetches tags with checkField scope", async () => {
    const tags = [
      { name: "title", label: "Page Title" },
      { name: "tag:og_image", label: "OG Image" },
    ];
    mockedListTags.mockResolvedValue(tags as never);

    const { result } = renderHook(() => useCheckFieldTags(), { wrapper });

    await waitFor(() => expect(result.current.tags).toHaveLength(2));
    expect(mockedListTags).toHaveBeenCalledWith("checkField");
  });

  it("returns empty while loading", () => {
    mockedListTags.mockReturnValue(new Promise(() => {}));
    const { result } = renderHook(() => useCheckFieldTags(), { wrapper });

    expect(result.current.tags).toEqual([]);
    expect(result.current.isLoading).toBe(true);
  });
});
