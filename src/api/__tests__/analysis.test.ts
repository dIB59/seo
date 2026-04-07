import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    startAnalysis: vi.fn(),
    getResult: vi.fn(),
  },
}));

import { commands } from "@/src/bindings";
import { startAnalysis, getResult, wrapTauriCommand } from "@/src/api/analysis";

describe("api/analysis", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("wrapTauriCommand returns Ok for { status: 'ok' }", async () => {
    const r = await wrapTauriCommand(Promise.resolve({ status: "ok", data: 123 }));
    expect(r.isOk()).toBe(true);
    expect(r.unwrap()).toBe(123);
  });

  it("wrapTauriCommand returns Err for { status: 'error' }", async () => {
    const r = await wrapTauriCommand(Promise.resolve({ status: "error", error: "bad" }));
    expect(r.isErr()).toBe(true);
    expect(r.unwrapErr()).toBe("bad");
  });

  it("startAnalysis forwards args and unwraps the response", async () => {
    (commands.startAnalysis as ReturnType<typeof vi.fn>).mockResolvedValue({
      status: "ok",
      data: { jobId: "abc" },
    });
    const r = await startAnalysis("https://example.com", null as never);
    expect(commands.startAnalysis).toHaveBeenCalledWith("https://example.com", null);
    expect(r.isOk()).toBe(true);
    expect(r.unwrap()).toEqual({ jobId: "abc" });
  });

  it("getResult propagates error responses", async () => {
    (commands.getResult as ReturnType<typeof vi.fn>).mockResolvedValue({
      status: "error",
      error: "missing",
    });
    const r = await getResult("job-1");
    expect(r.isErr()).toBe(true);
    expect(r.unwrapErr()).toBe("missing");
  });
});
