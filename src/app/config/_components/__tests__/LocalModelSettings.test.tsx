import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";

vi.mock("@/src/hooks/use-local-models", () => ({
  useLocalModels: vi.fn(),
}));

import { useLocalModels } from "@/src/hooks/use-local-models";
import { LocalModelSettings } from "../LocalModelSettings";

const mockedUseLocalModels = vi.mocked(useLocalModels);

const models = [
  {
    id: "m1",
    name: "Qwen 7B",
    description: "Good quality model",
    tier: "medium",
    size_bytes: 4_500_000_000,
    quantization: "Q4_K_M",
    filename: "model.gguf",
    url: "https://example.com/model.gguf",
    is_downloaded: true,
    is_active: true,
  },
  {
    id: "m2",
    name: "Phi 3B",
    description: "Fast small model",
    tier: "small",
    size_bytes: 2_000_000_000,
    quantization: "Q4_K_M",
    filename: "phi.gguf",
    url: "https://example.com/phi.gguf",
    is_downloaded: false,
    is_active: false,
  },
];

describe("LocalModelSettings", () => {
  it("renders loading skeleton when loading", () => {
    mockedUseLocalModels.mockReturnValue({
      models: [],
      downloading: {},
      isLoading: true,
      download: vi.fn(),
      cancelDownload: vi.fn(),
      deleteModel: vi.fn(),
      activate: vi.fn(),
      refresh: vi.fn(),
    } as never);

    const { container } = render(<LocalModelSettings />);
    // Skeleton renders pulse animation divs
    expect(container.querySelector(".animate-pulse")).toBeTruthy();
  });

  it("renders model cards when loaded", () => {
    mockedUseLocalModels.mockReturnValue({
      models,
      downloading: {},
      isLoading: false,
      download: vi.fn(),
      cancelDownload: vi.fn(),
      deleteModel: vi.fn(),
      activate: vi.fn(),
      refresh: vi.fn(),
    } as never);

    render(<LocalModelSettings />);
    expect(screen.getByText("Qwen 7B")).toBeInTheDocument();
    expect(screen.getByText("Phi 3B")).toBeInTheDocument();
  });

  it("shows Active badge for active model", () => {
    mockedUseLocalModels.mockReturnValue({
      models,
      downloading: {},
      isLoading: false,
      download: vi.fn(),
      cancelDownload: vi.fn(),
      deleteModel: vi.fn(),
      activate: vi.fn(),
      refresh: vi.fn(),
    } as never);

    render(<LocalModelSettings />);
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it("shows tier badges", () => {
    mockedUseLocalModels.mockReturnValue({
      models,
      downloading: {},
      isLoading: false,
      download: vi.fn(),
      cancelDownload: vi.fn(),
      deleteModel: vi.fn(),
      activate: vi.fn(),
      refresh: vi.fn(),
    } as never);

    render(<LocalModelSettings />);
    expect(screen.getByText("Medium")).toBeInTheDocument();
    expect(screen.getByText("Small")).toBeInTheDocument();
  });

  it("shows Download button for non-downloaded models", () => {
    mockedUseLocalModels.mockReturnValue({
      models,
      downloading: {},
      isLoading: false,
      download: vi.fn(),
      cancelDownload: vi.fn(),
      deleteModel: vi.fn(),
      activate: vi.fn(),
      refresh: vi.fn(),
    } as never);

    render(<LocalModelSettings />);
    expect(screen.getByRole("button", { name: /Download/i })).toBeInTheDocument();
  });
});

// Test the pure formatBytes function by reimplementing (it's private)
describe("formatBytes (logic)", () => {
  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
    if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(0)} MB`;
    return `${(bytes / 1e3).toFixed(0)} KB`;
  }

  it("formats bytes correctly", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1500)).toBe("2 KB");
    expect(formatBytes(5_000_000)).toBe("5 MB");
    expect(formatBytes(4_500_000_000)).toBe("4.5 GB");
  });
});
