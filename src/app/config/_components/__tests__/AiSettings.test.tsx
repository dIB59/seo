import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("@/src/bindings", () => ({
  commands: {
    getAiSource: vi.fn(),
    setAiSource: vi.fn(),
    getGeminiApiKey: vi.fn(),
  },
}));

vi.mock("@/src/api/ai", () => ({
  set_gemini_api_key: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("../LocalModelSettings", () => ({
  LocalModelSettings: () => <div data-testid="local-model-settings">Local Model</div>,
}));

import { commands } from "@/src/bindings";
import { AiSettings } from "../AiSettings";

const mocked = vi.mocked(commands);

beforeEach(() => {
  vi.clearAllMocks();
  mocked.getAiSource.mockResolvedValue({ status: "ok", data: "gemini" } as never);
  mocked.getGeminiApiKey.mockResolvedValue({ status: "ok", data: "test-key" } as never);
  mocked.setAiSource.mockResolvedValue({ status: "ok", data: null } as never);
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
      expect(mocked.setAiSource).toHaveBeenCalledWith("local");
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
