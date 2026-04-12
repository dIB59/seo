import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/src/bindings", () => ({
  commands: {
    getGeminiApiKey: vi.fn(),
    setGeminiApiKey: vi.fn(),
    getGeminiPersona: vi.fn(),
    setGeminiPersona: vi.fn(),
    getGeminiEnabled: vi.fn(),
    setGeminiEnabled: vi.fn(),
    getGeminiPromptBlocks: vi.fn(),
    setGeminiPromptBlocks: vi.fn(),
    getGeminiInsights: vi.fn(),
    getAiSource: vi.fn(),
    generateLocalInsights: vi.fn(),
  },
}));

// Mock the settings dialog event
vi.stubGlobal("window", {
  ...globalThis.window,
  dispatchEvent: vi.fn(),
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
});

vi.mock("sonner", () => ({
  toast: vi.fn().mockReturnValue(undefined),
}));

import { commands } from "@/src/bindings";
import {
  getApiKey,
  setApiKey,
  getPersona,
  setPersona,
  getAiEnabled,
  setAiEnabled,
} from "../ai";

const mocked = vi.mocked(commands);
beforeEach(() => vi.clearAllMocks());

describe("getApiKey", () => {
  it("returns Ok with key on success", async () => {
    mocked.getGeminiApiKey.mockResolvedValue({ status: "ok", data: "AIza..." } as never);
    const result = await getApiKey();
    expect(result.isOk()).toBe(true);
    expect(result.unwrap()).toBe("AIza...");
  });

  it("returns Err on failure", async () => {
    mocked.getGeminiApiKey.mockResolvedValue({ status: "error", error: "Not found" } as never);
    const result = await getApiKey();
    expect(result.isErr()).toBe(true);
  });
});

describe("setApiKey", () => {
  it("returns Ok on success", async () => {
    mocked.setGeminiApiKey.mockResolvedValue({ status: "ok", data: null } as never);
    const result = await setApiKey("new-key");
    expect(result.isOk()).toBe(true);
    expect(mocked.setGeminiApiKey).toHaveBeenCalledWith("new-key");
  });
});

describe("getPersona", () => {
  it("returns persona text", async () => {
    mocked.getGeminiPersona.mockResolvedValue({ status: "ok", data: "Be concise" } as never);
    const result = await getPersona();
    expect(result.unwrap()).toBe("Be concise");
  });
});

describe("setPersona", () => {
  it("saves persona", async () => {
    mocked.setGeminiPersona.mockResolvedValue({ status: "ok", data: null } as never);
    const result = await setPersona("New persona");
    expect(result.isOk()).toBe(true);
  });
});

describe("getAiEnabled", () => {
  it("returns boolean", async () => {
    mocked.getGeminiEnabled.mockResolvedValue({ status: "ok", data: true } as never);
    const result = await getAiEnabled();
    expect(result.unwrap()).toBe(true);
  });
});

describe("setAiEnabled", () => {
  it("sets enabled state", async () => {
    mocked.setGeminiEnabled.mockResolvedValue({ status: "ok", data: null } as never);
    const result = await setAiEnabled(false);
    expect(result.isOk()).toBe(true);
    expect(mocked.setGeminiEnabled).toHaveBeenCalledWith(false);
  });
});
