import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { SWRConfig } from "swr";
import type { ReactNode } from "react";
import { useCrudState } from "../use-crud-state";

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

import { toast } from "sonner";

interface Item {
  id: string;
  name: string;
}

type Form = { name: string };

function wrapper({ children }: { children: ReactNode }) {
  return (
    <SWRConfig value={{ dedupingInterval: 0, provider: () => new Map() }}>
      {children}
    </SWRConfig>
  );
}

const items: Item[] = [
  { id: "1", name: "Alpha" },
  { id: "2", name: "Beta" },
];

const mockFetcher = vi.fn().mockResolvedValue(items);
const mockCreate = vi.fn().mockResolvedValue({ id: "3", name: "Gamma" });
const mockUpdate = vi.fn().mockResolvedValue({ id: "1", name: "Updated" });
const mockDelete = vi.fn().mockResolvedValue(undefined);

beforeEach(() => vi.clearAllMocks());

function renderCrud() {
  return renderHook(
    () =>
      useCrudState<Item, Form>({
        swrKey: "test-items",
        fetcher: mockFetcher,
        onCreate: mockCreate,
        onUpdate: mockUpdate,
        onDelete: mockDelete,
        entityName: "Item",
      }),
    { wrapper },
  );
}

describe("useCrudState", () => {
  it("starts with dialog closed", () => {
    const { result } = renderCrud();
    expect(result.current.dialogOpen).toBe(false);
    expect(result.current.editing).toBeNull();
    expect(result.current.saving).toBe(false);
  });

  it("openCreate opens dialog with null editing", () => {
    const { result } = renderCrud();
    act(() => result.current.openCreate());
    expect(result.current.dialogOpen).toBe(true);
    expect(result.current.editing).toBeNull();
  });

  it("openEdit opens dialog with the item", () => {
    const { result } = renderCrud();
    act(() => result.current.openEdit(items[0]));
    expect(result.current.dialogOpen).toBe(true);
    expect(result.current.editing).toEqual(items[0]);
  });

  it("handleSave creates when not editing", async () => {
    const { result } = renderCrud();

    act(() => result.current.openCreate());

    await act(async () => {
      await result.current.handleSave({ name: "Gamma" });
    });

    expect(mockCreate).toHaveBeenCalledWith({ name: "Gamma" });
    expect(toast.success).toHaveBeenCalledWith("Item created");
    expect(result.current.dialogOpen).toBe(false);
  });

  it("handleSave updates when editing", async () => {
    const { result } = renderCrud();

    act(() => result.current.openEdit(items[0]));

    await act(async () => {
      await result.current.handleSave({ name: "Updated" });
    });

    expect(mockUpdate).toHaveBeenCalledWith("1", { name: "Updated" });
    expect(toast.success).toHaveBeenCalledWith("Item updated");
  });

  it("handleDelete removes the item", async () => {
    const { result } = renderCrud();

    await act(async () => {
      await result.current.handleDelete("2");
    });

    expect(mockDelete).toHaveBeenCalledWith("2");
    expect(toast.success).toHaveBeenCalledWith("Item deleted");
  });

  it("shows error toast on save failure", async () => {
    mockCreate.mockRejectedValueOnce(new Error("DB error"));
    const { result } = renderCrud();

    act(() => result.current.openCreate());

    await act(async () => {
      await result.current.handleSave({ name: "Fail" });
    });

    expect(toast.error).toHaveBeenCalledWith("DB error");
    // Dialog stays open on error
    expect(result.current.dialogOpen).toBe(true);
  });
});
