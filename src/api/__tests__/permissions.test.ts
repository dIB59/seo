import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    getUserPolicy: vi.fn(),
  },
}));

import { commands } from "@/src/bindings";
import { getUserPolicy } from "../permissions";

const mocked = vi.mocked(commands);
beforeEach(() => vi.clearAllMocks());

describe("getUserPolicy", () => {
  it("returns Ok with policy on success", async () => {
    const policy = { tier: "Free", max_pages: 10, enabled_features: ["LinkAnalysis"] };
    mocked.getUserPolicy.mockResolvedValue({ status: "ok", data: policy } as never);

    const result = await getUserPolicy();
    expect(result.isOk()).toBe(true);
    expect(result.unwrap()).toEqual(policy);
  });

  it("returns Err on failure", async () => {
    mocked.getUserPolicy.mockResolvedValue({ status: "error", error: "Network error" } as never);

    const result = await getUserPolicy();
    expect(result.isErr()).toBe(true);
    expect(result.unwrapErr()).toBe("Network error");
  });
});
