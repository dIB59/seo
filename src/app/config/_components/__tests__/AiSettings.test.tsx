import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// Mock the API layer (not raw commands — component now uses API functions)
vi.mock("@/src/api/ai", () => ({
  getAiSource: vi.fn(),
  setAiSource: vi.fn(),
  getApiKey: vi.fn(),
  setApiKey: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("../LocalModelSettings", () => ({
  LocalModelSettings: () => <div data-testid="local-model-settings">Local Model</div>,
}));

import {
  getAiSource,
  setAiSource,
  getApiKey,
} from "@/src/api/ai";
import { AiSettings } from "../AiSettings";

const mockedGetSource = vi.mocked(getAiSource);
const mockedSetSource = vi.mocked(setAiSource);
const mockedGetKey = vi.mocked(getApiKey);

// Minimal Result-like objects
const ok = <T,>(data: T) => ({ isOk: () => true, isErr: () => false, unwrap: () => data });
const err = (msg: string) => ({ isOk: () => false, isErr: () => true, unwrapErr: () => msg });

beforeEach(() => {
  vi.clearAllMocks();
  mockedGetSource.mockResolvedValue(ok("gemini") as never);
  mockedGetKey.mockResolvedValue(ok("test-key") as never);
  mockedSetSource.mockResolvedValue(ok(null) as never);
});

describe("AiSettings", () => {
  it("renders source picker with Gemini and Local options", async () => {
    render(<AiSettings />);

    await waitFor(() => {
      expect(screen.getByText("Gemini (Cloud)")).toBeInTheDocument();
      expect(screen.getByText("Local Model (On-device)")).toBeInTheDocument();
    });
  });

  it("shows Gemini API key input when Gemini is selected", async () => {
    render(<AiSettings />);

    await waitFor(() => {
      expect(screen.getByText("Gemini API Key")).toBeInTheDocument();
      expect(screen.getByPlaceholderText("AIza...")).toBeInTheDocument();
    });
  });

  it("loads and displays the current API key", async () => {
    render(<AiSettings />);

    await waitFor(() => {
      const input = screen.getByPlaceholderText("AIza...") as HTMLInputElement;
      expect(input.value).toBe("test-key");
    });
  });

  it("switches to local model view on click", async () => {
    const user = userEvent.setup();
    render(<AiSettings />);

    await waitFor(() => screen.getByText("Local Model (On-device)"));

    await user.click(screen.getByText("Local Model (On-device)"));

    await waitFor(() => {
      expect(mockedSetSource).toHaveBeenCalledWith("local");
      expect(screen.getByTestId("local-model-settings")).toBeInTheDocument();
    });
  });

  it("has a Save Key button", async () => {
    render(<AiSettings />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Save Key/i })).toBeInTheDocument();
    });
  });
});
