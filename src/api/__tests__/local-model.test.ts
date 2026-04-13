import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    listLocalModels: vi.fn(),
    downloadLocalModel: vi.fn(),
    cancelModelDownload: vi.fn(),
    deleteLocalModel: vi.fn(),
    getActiveLocalModel: vi.fn(),
    setActiveLocalModel: vi.fn(),
  },
}));

import { commands } from "@/src/bindings";
import {
  listLocalModels,
  downloadLocalModel,
  deleteLocalModel,
  getActiveLocalModel,
  setActiveLocalModel,
} from "../local-model";

const mocked = vi.mocked(commands);
beforeEach(() => vi.clearAllMocks());

describe("listLocalModels", () => {
  it("returns models on success", async () => {
    const models = [{ id: "m1", name: "Qwen 7B" }];
    mocked.listLocalModels.mockResolvedValue({ status: "ok", data: models } as never);
    const result = await listLocalModels();
    expect(result.isOk()).toBe(true);
    expect(result.unwrap()).toEqual(models);
  });

  it("returns Err on failure", async () => {
    mocked.listLocalModels.mockResolvedValue({ status: "error", error: "IO error" } as never);
    const result = await listLocalModels();
    expect(result.isErr()).toBe(true);
  });
});

describe("downloadLocalModel", () => {
  it("returns Ok on success", async () => {
    mocked.downloadLocalModel.mockResolvedValue({ status: "ok", data: null } as never);
    const result = await downloadLocalModel("m1");
    expect(result.isOk()).toBe(true);
    expect(mocked.downloadLocalModel).toHaveBeenCalledWith("m1");
  });
});

describe("deleteLocalModel", () => {
  it("returns Ok on success", async () => {
    mocked.deleteLocalModel.mockResolvedValue({ status: "ok", data: null } as never);
    const result = await deleteLocalModel("m1");
    expect(result.isOk()).toBe(true);
  });
});

describe("getActiveLocalModel", () => {
  it("returns model id on success", async () => {
    mocked.getActiveLocalModel.mockResolvedValue({ status: "ok", data: "m1" } as never);
    const result = await getActiveLocalModel();
    expect(result.unwrap()).toBe("m1");
  });

  it("returns null when no model active", async () => {
    mocked.getActiveLocalModel.mockResolvedValue({ status: "ok", data: null } as never);
    const result = await getActiveLocalModel();
    expect(result.unwrap()).toBeNull();
  });
});

describe("setActiveLocalModel", () => {
  it("calls with model id", async () => {
    mocked.setActiveLocalModel.mockResolvedValue({ status: "ok", data: null } as never);
    const result = await setActiveLocalModel("m2");
    expect(result.isOk()).toBe(true);
    expect(mocked.setActiveLocalModel).toHaveBeenCalledWith("m2");
  });
});
