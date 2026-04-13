import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useFormSync } from "../use-form-sync";

interface Form {
  name: string;
  value: number;
}

const DEFAULTS: Form = { name: "", value: 0 };
const toForm = (item: { label: string; count: number }): Form => ({
  name: item.label,
  value: item.count,
});

describe("useFormSync", () => {
  it("initializes with defaults when open and no editing item", () => {
    const { result } = renderHook(() =>
      useFormSync(true, null, DEFAULTS, toForm),
    );
    expect(result.current[0]).toEqual(DEFAULTS);
  });

  it("initializes with editing item's values when open", () => {
    const editing = { label: "Test", count: 42 };
    const { result } = renderHook(() =>
      useFormSync(true, editing, DEFAULTS, toForm),
    );
    expect(result.current[0]).toEqual({ name: "Test", value: 42 });
  });

  it("resets to defaults when editing becomes null", () => {
    const editing = { label: "Test", count: 42 };
    const { result, rerender } = renderHook(
      ({ open, editing }) => useFormSync(open, editing, DEFAULTS, toForm),
      { initialProps: { open: true, editing: editing as typeof editing | null } },
    );

    expect(result.current[0].name).toBe("Test");

    rerender({ open: true, editing: null });
    expect(result.current[0]).toEqual(DEFAULTS);
  });

  it("allows manual updates via setForm", () => {
    const { result } = renderHook(() =>
      useFormSync(true, null, DEFAULTS, toForm),
    );

    act(() => {
      result.current[1]({ name: "Manual", value: 99 });
    });
    expect(result.current[0]).toEqual({ name: "Manual", value: 99 });
  });
});
