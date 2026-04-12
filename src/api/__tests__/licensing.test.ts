import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    activateWithKey: vi.fn(),
    getLicenseTier: vi.fn(),
    getMachineId: vi.fn(),
  },
}));

import { commands } from "@/src/bindings";
import { activateLicense, getLicenseTier, getMachineId } from "../licensing";

const mocked = vi.mocked(commands);
beforeEach(() => vi.clearAllMocks());

describe("activateLicense", () => {
  it("returns Ok with policy on success", async () => {
    const policy = { tier: "Premium", max_pages: 10000, enabled_features: [] };
    mocked.activateWithKey.mockResolvedValue({ status: "ok", data: policy } as never);

    const result = await activateLicense("VALID-KEY");
    expect(result.isOk()).toBe(true);
    expect(result.unwrap()).toEqual(policy);
  });

  it("returns Err on failure", async () => {
    mocked.activateWithKey.mockResolvedValue({ status: "error", error: "Invalid key" } as never);

    const result = await activateLicense("BAD-KEY");
    expect(result.isErr()).toBe(true);
    expect(result.unwrapErr()).toBe("Invalid key");
  });
});

describe("getLicenseTier", () => {
  it("returns the tier string on success", async () => {
    mocked.getLicenseTier.mockResolvedValue({ status: "ok", data: "Premium" } as never);

    const result = await getLicenseTier();
    expect(result.unwrap()).toBe("Premium");
  });
});

describe("getMachineId", () => {
  it("returns the machine ID on success", async () => {
    mocked.getMachineId.mockResolvedValue({ status: "ok", data: "abc-123" } as never);

    const result = await getMachineId();
    expect(result.unwrap()).toBe("abc-123");
  });

  it("returns Err on failure", async () => {
    mocked.getMachineId.mockResolvedValue({ status: "error", error: "Timeout" } as never);

    const result = await getMachineId();
    expect(result.isErr()).toBe(true);
  });
});
