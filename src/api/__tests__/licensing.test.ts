import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    activateWithKey: vi.fn(),
    getLicenseTier: vi.fn(),
    getMachineId: vi.fn(),
  },
}));

import { commands } from "@/src/bindings";
import { activate_license, get_license_tier, get_machine_id } from "../licensing";

const mocked = vi.mocked(commands);
beforeEach(() => vi.clearAllMocks());

describe("activate_license", () => {
  it("returns Ok with policy on success", async () => {
    const policy = { tier: "Premium", max_pages: 10000, enabled_features: [] };
    mocked.activateWithKey.mockResolvedValue({ status: "ok", data: policy } as never);

    const result = await activate_license("VALID-KEY");
    expect(result.isOk()).toBe(true);
    expect(result.unwrap()).toEqual(policy);
  });

  it("returns Err on failure", async () => {
    mocked.activateWithKey.mockResolvedValue({ status: "error", error: "Invalid key" } as never);

    const result = await activate_license("BAD-KEY");
    expect(result.isErr()).toBe(true);
    expect(result.unwrapErr()).toBe("Invalid key");
  });
});

describe("get_license_tier", () => {
  it("returns the tier string on success", async () => {
    mocked.getLicenseTier.mockResolvedValue({ status: "ok", data: "Premium" } as never);

    const result = await get_license_tier();
    expect(result.unwrap()).toBe("Premium");
  });
});

describe("get_machine_id", () => {
  it("returns the machine ID on success", async () => {
    mocked.getMachineId.mockResolvedValue({ status: "ok", data: "abc-123" } as never);

    const result = await get_machine_id();
    expect(result.unwrap()).toBe("abc-123");
  });

  it("returns Err on failure", async () => {
    mocked.getMachineId.mockResolvedValue({ status: "error", error: "Timeout" } as never);

    const result = await get_machine_id();
    expect(result.isErr()).toBe(true);
  });
});
