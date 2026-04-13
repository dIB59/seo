import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { JobFilterBar } from "../organisms/JobFilterBar";

const defaultProps = {
  total: 42,
  urlFilter: "",
  setUrlFilter: vi.fn(),
  statusFilter: "all",
  setStatusFilter: vi.fn(),
  pageSize: 5,
  setPageSize: vi.fn(),
  setCurrentPage: vi.fn(),
};

describe("JobFilterBar", () => {
  it("renders the URL filter input", () => {
    render(<JobFilterBar {...defaultProps} />);
    expect(screen.getByPlaceholderText("Filter by URL...")).toBeInTheDocument();
  });

  it("shows total count", () => {
    render(<JobFilterBar {...defaultProps} />);
    expect(screen.getByText("42")).toBeInTheDocument();
  });

  it("calls setUrlFilter and resets page on input", async () => {
    const setUrlFilter = vi.fn();
    const setCurrentPage = vi.fn();
    const user = userEvent.setup();

    render(
      <JobFilterBar
        {...defaultProps}
        setUrlFilter={setUrlFilter}
        setCurrentPage={setCurrentPage}
      />,
    );

    await user.type(screen.getByPlaceholderText("Filter by URL..."), "example");
    expect(setUrlFilter).toHaveBeenCalled();
    expect(setCurrentPage).toHaveBeenCalledWith(1);
  });

  it("shows Reset button when filters are active", () => {
    render(<JobFilterBar {...defaultProps} urlFilter="test" />);
    expect(screen.getByRole("button", { name: /Reset/i })).toBeInTheDocument();
  });

  it("hides Reset button when no filters active", () => {
    render(<JobFilterBar {...defaultProps} />);
    expect(screen.queryByRole("button", { name: /Reset/i })).not.toBeInTheDocument();
  });

  it("clears filters on Reset click", async () => {
    const setUrlFilter = vi.fn();
    const setStatusFilter = vi.fn();
    const setCurrentPage = vi.fn();
    const user = userEvent.setup();

    render(
      <JobFilterBar
        {...defaultProps}
        urlFilter="example"
        setUrlFilter={setUrlFilter}
        setStatusFilter={setStatusFilter}
        setCurrentPage={setCurrentPage}
      />,
    );

    await user.click(screen.getByRole("button", { name: /Reset/i }));
    expect(setUrlFilter).toHaveBeenCalledWith("");
    expect(setStatusFilter).toHaveBeenCalledWith("all");
    expect(setCurrentPage).toHaveBeenCalledWith(1);
  });

  it("hides total when count is 0", () => {
    render(<JobFilterBar {...defaultProps} total={0} />);
    expect(screen.queryByText("Total:")).not.toBeInTheDocument();
  });
});
