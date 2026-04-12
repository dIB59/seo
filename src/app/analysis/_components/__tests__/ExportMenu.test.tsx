import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ExportMenu } from "../molecules/ExportMenu";

describe("ExportMenu", () => {
  const handlers = {
    onPDF: vi.fn(),
    onText: vi.fn(),
    onCSV: vi.fn(),
  };

  it("renders the Export Report button", () => {
    render(<ExportMenu {...handlers} />);
    expect(screen.getByRole("button", { name: /Export Report/i })).toBeInTheDocument();
  });

  it("shows dropdown items on click", async () => {
    const user = userEvent.setup();
    render(<ExportMenu {...handlers} />);

    await user.click(screen.getByRole("button", { name: /Export Report/i }));

    expect(screen.getByText("Download PDF")).toBeInTheDocument();
    expect(screen.getByText("Download Text Report")).toBeInTheDocument();
    expect(screen.getByText("Download CSV Data")).toBeInTheDocument();
  });

  it("calls onPDF when PDF item clicked", async () => {
    const user = userEvent.setup();
    render(<ExportMenu {...handlers} />);

    await user.click(screen.getByRole("button", { name: /Export Report/i }));
    await user.click(screen.getByText("Download PDF"));

    expect(handlers.onPDF).toHaveBeenCalledTimes(1);
  });

  it("calls onCSV when CSV item clicked", async () => {
    const user = userEvent.setup();
    render(<ExportMenu {...handlers} />);

    await user.click(screen.getByRole("button", { name: /Export Report/i }));
    await user.click(screen.getByText("Download CSV Data"));

    expect(handlers.onCSV).toHaveBeenCalledTimes(1);
  });
});
