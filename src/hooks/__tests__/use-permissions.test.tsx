import { describe, it, expect, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";

vi.mock("@/src/api/permissions", () => ({
  getUserPolicy: vi.fn(),
}));
vi.mock("@/src/api/licensing", () => ({
  getMachineId: vi.fn(),
}));

import { getUserPolicy } from "@/src/api/permissions";
import { getMachineId } from "@/src/api/licensing";
import { usePermissions } from "../use-permissions";

const mockedGetPolicy = vi.mocked(getUserPolicy);
const mockedGetMachineId = vi.mocked(getMachineId);

function wrapper({ children }: { children: ReactNode }) {
  return (
    <SWRConfig value={{ dedupingInterval: 0, provider: () => new Map() }}>
      {children}
    </SWRConfig>
  );
}

const freePolicy = {
  tier: "Free" as const,
  max_pages: 10,
  enabled_features: ["LinkAnalysis"] as string[],
};

const premiumPolicy = {
  tier: "Premium" as const,
  max_pages: 10000,
  enabled_features: ["LinkAnalysis", "DeepAudit", "ReportExport"] as string[],
};

// Minimal Result-like objects
const ok = <T,>(data: T) => ({ isOk: () => true, unwrap: () => data });
const err = (msg: string) => ({ isOk: () => false, unwrapErr: () => msg });

describe("usePermissions", () => {
  it("returns free user status for free tier", async () => {
    mockedGetPolicy.mockResolvedValue(ok(freePolicy) as never);
    mockedGetMachineId.mockResolvedValue(ok("machine-123") as never);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => expect(result.current.policy).toBeDefined());

    expect(result.current.isFreeUser).toBe(true);
    expect(result.current.isPremiumUser).toBe(false);
    expect(result.current.maxPages).toBe(10);
    expect(result.current.machineId).toBe("machine-123");
  });

  it("returns premium user status for premium tier", async () => {
    mockedGetPolicy.mockResolvedValue(ok(premiumPolicy) as never);
    mockedGetMachineId.mockResolvedValue(ok("machine-456") as never);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => expect(result.current.policy).toBeDefined());

    expect(result.current.isFreeUser).toBe(false);
    expect(result.current.isPremiumUser).toBe(true);
    expect(result.current.maxPages).toBe(10000);
  });

  it("hasFeature returns true for enabled features", async () => {
    mockedGetPolicy.mockResolvedValue(ok(premiumPolicy) as never);
    mockedGetMachineId.mockResolvedValue(ok("m") as never);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => expect(result.current.policy).toBeDefined());

    expect(result.current.hasFeature("LinkAnalysis" as never)).toBe(true);
    expect(result.current.hasFeature("DeepAudit" as never)).toBe(true);
    expect(result.current.hasFeature("NonExistent" as never)).toBe(false);
  });

  it("canAnalyzePages respects max_pages", async () => {
    mockedGetPolicy.mockResolvedValue(ok(freePolicy) as never);
    mockedGetMachineId.mockResolvedValue(ok("m") as never);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => expect(result.current.policy).toBeDefined());

    expect(result.current.canAnalyzePages(5)).toBe(true);
    expect(result.current.canAnalyzePages(10)).toBe(true);
    expect(result.current.canAnalyzePages(11)).toBe(false);
  });

  it("handles API errors gracefully", async () => {
    mockedGetPolicy.mockResolvedValue(err("network error") as never);
    mockedGetMachineId.mockResolvedValue(err("timeout") as never);

    const { result } = renderHook(() => usePermissions(), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.policy).toBeUndefined();
    expect(result.current.machineId).toBe("");
    expect(result.current.isFreeUser).toBe(false);
    expect(result.current.isPremiumUser).toBe(false);
    expect(result.current.maxPages).toBe(1); // fallback
  });
});
