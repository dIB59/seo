import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useMutation } from "../use-mutation";

// Suppress sonner toasts in tests
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

describe("useMutation", () => {
  it("starts with isLoading false and no error", () => {
    const { result } = renderHook(() =>
      useMutation(async () => "ok"),
    );
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("sets isLoading during execution", async () => {
    let resolve: (v: string) => void;
    const fn = () => new Promise<string>((r) => { resolve = r; });

    const { result } = renderHook(() => useMutation(fn));

    let promise: Promise<unknown>;
    act(() => {
      promise = result.current.execute();
    });
    expect(result.current.isLoading).toBe(true);

    await act(async () => {
      resolve!("done");
      await promise;
    });
    expect(result.current.isLoading).toBe(false);
  });

  it("returns the result on success", async () => {
    const fn = async (x: number) => x * 2;
    const { result } = renderHook(() => useMutation(fn));

    let value: number | undefined;
    await act(async () => {
      value = await result.current.execute(5);
    });
    expect(value).toBe(10);
  });

  it("sets error on failure", async () => {
    const fn = async () => { throw new Error("boom"); };
    const { result } = renderHook(() => useMutation(fn));

    await act(async () => {
      await result.current.execute();
    });
    expect(result.current.error).toBe("boom");
  });

  it("calls onSuccess callback", async () => {
    const onSuccess = vi.fn();
    const fn = async () => 42;
    const { result } = renderHook(() =>
      useMutation(fn, { onSuccess }),
    );

    await act(async () => {
      await result.current.execute();
    });
    expect(onSuccess).toHaveBeenCalledWith(42);
  });
});
