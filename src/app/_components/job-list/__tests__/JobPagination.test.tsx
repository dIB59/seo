import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { JobPagination } from "../molecules/JobPagination";

describe("JobPagination", () => {
  it("renders nothing when totalPages <= 1", () => {
    const { container } = render(
      <JobPagination currentPage={1} totalPages={1} onPageChange={vi.fn()} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders page numbers for small page counts", () => {
    render(
      <JobPagination currentPage={1} totalPages={3} onPageChange={vi.fn()} />,
    );
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
  });

  it("calls onPageChange when a page link is clicked", async () => {
    const onPageChange = vi.fn();
    const user = userEvent.setup();
    render(
      <JobPagination currentPage={1} totalPages={5} onPageChange={onPageChange} />,
    );

    await user.click(screen.getByText("3"));
    expect(onPageChange).toHaveBeenCalledWith(3);
  });

  it("disables Previous on first page", () => {
    render(
      <JobPagination currentPage={1} totalPages={5} onPageChange={vi.fn()} />,
    );
    const prev = screen.getByText("Previous").closest("a");
    expect(prev?.className).toContain("pointer-events-none");
  });

  it("disables Next on last page", () => {
    render(
      <JobPagination currentPage={5} totalPages={5} onPageChange={vi.fn()} />,
    );
    const next = screen.getByText("Next").closest("a");
    expect(next?.className).toContain("pointer-events-none");
  });

  it("navigates to previous page", async () => {
    const onPageChange = vi.fn();
    const user = userEvent.setup();
    render(
      <JobPagination currentPage={3} totalPages={5} onPageChange={onPageChange} />,
    );

    await user.click(screen.getByText("Previous"));
    expect(onPageChange).toHaveBeenCalledWith(2);
  });

  it("navigates to next page", async () => {
    const onPageChange = vi.fn();
    const user = userEvent.setup();
    render(
      <JobPagination currentPage={3} totalPages={5} onPageChange={onPageChange} />,
    );

    await user.click(screen.getByText("Next"));
    expect(onPageChange).toHaveBeenCalledWith(4);
  });
});
