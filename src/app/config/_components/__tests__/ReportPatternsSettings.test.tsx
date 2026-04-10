import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("swr", () => ({
  default: vi.fn(),
}));

vi.mock("@/src/api/report", () => ({
  listReportPatterns: vi.fn(),
  createReportPattern: vi.fn(),
  updateReportPattern: vi.fn(),
  toggleReportPattern: vi.fn(),
  deleteReportPattern: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

vi.mock("../ReportPatternDialog", () => ({
  ReportPatternDialog: ({ open }: { open: boolean }) =>
    open ? <div data-testid="pattern-dialog">Dialog</div> : null,
}));

vi.mock("../ReportPatternRow", () => ({
  ReportPatternRow: ({ pattern }: { pattern: { id: string; name: string } }) => (
    <tr data-testid={`pattern-row-${pattern.id}`}>
      <td>{pattern.name}</td>
    </tr>
  ),
}));

import useSWR from "swr";
import { ReportPatternsSettings } from "../ReportPatternsSettings";

const mockedUseSWR = vi.mocked(useSWR);

beforeEach(() => vi.clearAllMocks());

describe("ReportPatternsSettings", () => {
  it("shows empty state when no patterns", () => {
    mockedUseSWR.mockReturnValue({
      data: [],
      mutate: vi.fn(),
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    render(<ReportPatternsSettings />);
    expect(screen.getByText("No patterns configured.")).toBeInTheDocument();
  });

  it("renders pattern rows", () => {
    mockedUseSWR.mockReturnValue({
      data: [
        { id: "p1", name: "Missing Title" },
        { id: "p2", name: "Slow Pages" },
      ],
      mutate: vi.fn(),
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    render(<ReportPatternsSettings />);
    expect(screen.getByTestId("pattern-row-p1")).toBeInTheDocument();
    expect(screen.getByText("Missing Title")).toBeInTheDocument();
    expect(screen.getByText("Slow Pages")).toBeInTheDocument();
  });

  it("opens dialog on Add Pattern click", async () => {
    mockedUseSWR.mockReturnValue({
      data: [],
      mutate: vi.fn(),
      isLoading: false,
      isValidating: false,
      error: undefined,
    } as never);

    const user = userEvent.setup();
    render(<ReportPatternsSettings />);

    await user.click(screen.getByRole("button", { name: /Add Pattern/i }));
    expect(screen.getByTestId("pattern-dialog")).toBeInTheDocument();
  });
});
