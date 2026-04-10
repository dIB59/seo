import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    listReportPatterns: vi.fn(),
    createReportPattern: vi.fn(),
    listReportTemplates: vi.fn(),
    getReportTemplate: vi.fn(),
    createReportTemplate: vi.fn(),
    updateReportTemplate: vi.fn(),
    setActiveReportTemplate: vi.fn(),
    deleteReportTemplate: vi.fn(),
    generateReportData: vi.fn(),
  },
}));

import { commands } from "@/src/bindings";
import {
  listReportPatterns,
  listReportTemplates,
  getReportTemplate,
  updateReportTemplate,
  generateReportData,
} from "../report";

const mocked = vi.mocked(commands);

beforeEach(() => vi.clearAllMocks());

describe("listReportPatterns", () => {
  it("returns patterns on success", async () => {
    const patterns = [{ id: "p1", name: "Missing Title" }];
    mocked.listReportPatterns.mockResolvedValue({ status: "ok", data: patterns } as never);

    const result = await listReportPatterns();
    expect(result).toEqual(patterns);
  });

  it("throws on error", async () => {
    mocked.listReportPatterns.mockResolvedValue({
      status: "error",
      error: "DB error",
    } as never);

    await expect(listReportPatterns()).rejects.toThrow("DB error");
  });
});

describe("listReportTemplates", () => {
  it("returns templates on success", async () => {
    const templates = [{ id: "default", name: "Default Report" }];
    mocked.listReportTemplates.mockResolvedValue({ status: "ok", data: templates } as never);

    const result = await listReportTemplates();
    expect(result).toEqual(templates);
  });
});

describe("getReportTemplate", () => {
  it("fetches a single template by id", async () => {
    const template = { id: "default", name: "Default Report", sections: [] };
    mocked.getReportTemplate.mockResolvedValue({ status: "ok", data: template } as never);

    const result = await getReportTemplate("default");
    expect(result).toEqual(template);
    expect(mocked.getReportTemplate).toHaveBeenCalledWith("default");
  });
});

describe("updateReportTemplate", () => {
  it("calls update command", async () => {
    mocked.updateReportTemplate.mockResolvedValue({ status: "ok", data: null } as never);

    const template = { id: "t1", name: "Test", isBuiltin: false, sections: [], selectedTags: [] };
    await updateReportTemplate(template as never);
    expect(mocked.updateReportTemplate).toHaveBeenCalledWith(template);
  });

  it("throws on error", async () => {
    mocked.updateReportTemplate.mockResolvedValue({
      status: "error",
      error: "Not found",
    } as never);

    await expect(
      updateReportTemplate({ id: "x" } as never),
    ).rejects.toThrow("Not found");
  });
});

describe("generateReportData", () => {
  it("returns report data on success", async () => {
    const data = { jobId: "j1", seoScore: 85 };
    mocked.generateReportData.mockResolvedValue({ status: "ok", data } as never);

    const result = await generateReportData("j1");
    expect(result).toEqual(data);
  });
});
