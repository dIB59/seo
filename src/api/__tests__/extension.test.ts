import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    listTags: vi.fn(),
    listCustomChecks: vi.fn(),
    listCustomExtractors: vi.fn(),
    createCustomCheck: vi.fn(),
    createCustomExtractor: vi.fn(),
    deleteCustomCheck: vi.fn(),
    deleteCustomExtractor: vi.fn(),
  },
}));

import { commands } from "@/src/bindings";
import {
  listTags,
  listCustomChecks,
  listCustomExtractors,
  createCustomCheck,
} from "../extension";

const mocked = vi.mocked(commands);

beforeEach(() => vi.clearAllMocks());

describe("listTags", () => {
  it("returns tags on success", async () => {
    const tags = [{ name: "url", label: "Site URL" }];
    mocked.listTags.mockResolvedValue({ status: "ok", data: tags } as never);

    const result = await listTags();
    expect(result).toEqual(tags);
    expect(mocked.listTags).toHaveBeenCalledWith(null);
  });

  it("passes scope when provided", async () => {
    mocked.listTags.mockResolvedValue({ status: "ok", data: [] } as never);

    await listTags("checkField");
    expect(mocked.listTags).toHaveBeenCalledWith("checkField");
  });

  it("throws on error", async () => {
    mocked.listTags.mockResolvedValue({
      status: "error",
      error: "DB error",
    } as never);

    await expect(listTags()).rejects.toThrow("DB error");
  });
});

describe("listCustomChecks", () => {
  it("returns checks on success", async () => {
    const checks = [{ id: "1", name: "Test" }];
    mocked.listCustomChecks.mockResolvedValue({ status: "ok", data: checks } as never);

    const result = await listCustomChecks();
    expect(result).toEqual(checks);
  });
});

describe("listCustomExtractors", () => {
  it("returns extractors on success", async () => {
    const extractors = [{ id: "1", name: "OG Image", tag: "og_image" }];
    mocked.listCustomExtractors.mockResolvedValue({
      status: "ok",
      data: extractors,
    } as never);

    const result = await listCustomExtractors();
    expect(result).toEqual(extractors);
  });
});

describe("createCustomCheck", () => {
  it("returns created check on success", async () => {
    const check = { id: "new", name: "Test Check" };
    mocked.createCustomCheck.mockResolvedValue({ status: "ok", data: check } as never);

    const params = {
      name: "Test Check",
      severity: "warning" as const,
      field: "title",
      operator: "missing" as const,
      threshold: null,
      message_template: "Missing title",
      enabled: true,
    };
    const result = await createCustomCheck(params);
    expect(result).toEqual(check);
  });
});
