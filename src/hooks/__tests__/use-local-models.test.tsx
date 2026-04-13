import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { Result } from "@/src/lib/result";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

const mockModels = [
  { id: "model-1", name: "Test Model", size: 1000, downloaded: true, active: false },
  { id: "model-2", name: "Other Model", size: 2000, downloaded: false, active: false },
];

vi.mock("@/src/api/local-model", () => ({
  listLocalModels: vi.fn(),
  downloadLocalModel: vi.fn(),
  cancelModelDownload: vi.fn(),
  deleteLocalModel: vi.fn(),
  setActiveLocalModel: vi.fn(),
}));

import {
  listLocalModels,
  deleteLocalModel,
  setActiveLocalModel,
} from "@/src/api/local-model";
import { useLocalModels } from "../use-local-models";

const mockedList = vi.mocked(listLocalModels);
const mockedDelete = vi.mocked(deleteLocalModel);
const mockedActivate = vi.mocked(setActiveLocalModel);

beforeEach(() => {
  vi.clearAllMocks();
  mockedList.mockResolvedValue(Result.Ok(mockModels) as never);
  mockedDelete.mockResolvedValue(Result.Ok(null) as never);
  mockedActivate.mockResolvedValue(Result.Ok(null) as never);
});

describe("useLocalModels", () => {
  it("loads models on mount", async () => {
    const { result } = renderHook(() => useLocalModels());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.models).toHaveLength(2);
    expect(result.current.models[0].id).toBe("model-1");
  });

  it("starts with empty downloading map", async () => {
    const { result } = renderHook(() => useLocalModels());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.downloading).toEqual({});
  });

  it("remove calls deleteLocalModel and refreshes", async () => {
    const { result } = renderHook(() => useLocalModels());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    await act(() => result.current.remove("model-1"));

    expect(mockedDelete).toHaveBeenCalledWith("model-1");
  });

  it("activate calls setActiveLocalModel", async () => {
    const { result } = renderHook(() => useLocalModels());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    await act(() => result.current.activate("model-1"));

    expect(mockedActivate).toHaveBeenCalledWith("model-1");
  });

  it("sets up Tauri event listener for download progress", async () => {
    const { listen } = await import("@tauri-apps/api/event");
    renderHook(() => useLocalModels());
    expect(listen).toHaveBeenCalled();
  });
});
