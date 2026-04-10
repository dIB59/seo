import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// Mock SWR and API
vi.mock("swr", () => ({
  default: vi.fn(),
}));

vi.mock("@/src/api/extension", () => ({
  listCustomChecks: vi.fn(),
  createCustomCheck: vi.fn(),
  updateCustomCheck: vi.fn(),
  deleteCustomCheck: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

// Mock child components
vi.mock("../CustomCheckDialog", () => ({
  CustomCheckDialog: ({ open }: { open: boolean }) =>
    open ? <div data-testid="check-dialog">Dialog</div> : null,
}));

vi.mock("../CustomCheckRow", () => ({
  CustomCheckRow: ({
    check,
    onEdit,
    onDelete,
  }: {
    check: { id: string; name: string };
    onEdit: (c: unknown) => void;
    onDelete: (id: string) => void;
  }) => (
    <tr data-testid={`check-row-${check.id}`}>
      <td>{check.name}</td>
      <td>
        <button onClick={() => onEdit(check)}>Edit</button>
        <button onClick={() => onDelete(check.id)}>Delete</button>
      </td>
    </tr>
  ),
}));

import useSWR from "swr";
import { CustomChecksSettings } from "../CustomChecksSettings";

const mockedUseSWR = vi.mocked(useSWR);
const mutate = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
});

describe("CustomChecksSettings", () => {
  it("shows empty state when no checks exist", () => {
    mockedUseSWR.mockReturnValue({
      data: [],
      mutate,
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    render(<CustomChecksSettings />);
    expect(screen.getByText("No custom checks configured.")).toBeInTheDocument();
  });

  it("renders check rows when checks exist", () => {
    mockedUseSWR.mockReturnValue({
      data: [
        { id: "c1", name: "Missing OG Image", severity: "warning" },
        { id: "c2", name: "Slow Pages", severity: "critical" },
      ],
      mutate,
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    render(<CustomChecksSettings />);
    expect(screen.getByTestId("check-row-c1")).toBeInTheDocument();
    expect(screen.getByTestId("check-row-c2")).toBeInTheDocument();
    expect(screen.getByText("Missing OG Image")).toBeInTheDocument();
  });

  it("opens dialog when Add Check is clicked", async () => {
    mockedUseSWR.mockReturnValue({
      data: [],
      mutate,
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    const user = userEvent.setup();
    render(<CustomChecksSettings />);

    await user.click(screen.getByRole("button", { name: /Add Check/i }));
    expect(screen.getByTestId("check-dialog")).toBeInTheDocument();
  });

  it("has an Add Check button", () => {
    mockedUseSWR.mockReturnValue({
      data: [],
      mutate,
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    render(<CustomChecksSettings />);
    expect(screen.getByRole("button", { name: /Add Check/i })).toBeInTheDocument();
  });
});
