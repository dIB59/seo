import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { OverviewTab } from "../molecules/OverviewTab";

const issues = [
  { severity: "critical" },
  { severity: "critical" },
  { severity: "warning" },
  { severity: "info" },
] as never[];

const pages = [
  { load_time: 0.5 },  // fast
  { load_time: 1.5 },  // medium
  { load_time: 3.0 },  // slow
  { load_time: 0.8 },  // fast
] as never[];

describe("OverviewTab", () => {
  it("renders Issue Distribution card", () => {
    render(<OverviewTab issues={issues} pages={pages} />);
    expect(screen.getByText("Issue Distribution")).toBeInTheDocument();
  });

  it("renders Performance Summary card", () => {
    render(<OverviewTab issues={issues} pages={pages} />);
    expect(screen.getByText("Performance Summary")).toBeInTheDocument();
  });

  it("shows issue severity labels", () => {
    render(<OverviewTab issues={issues} pages={pages} />);
    expect(screen.getByText("Critical")).toBeInTheDocument();
    expect(screen.getByText("Warning")).toBeInTheDocument();
    expect(screen.getByText("Suggestion")).toBeInTheDocument();
  });

  it("shows performance speed categories", () => {
    render(<OverviewTab issues={issues} pages={pages} />);
    expect(screen.getByText("Fast (<1s)")).toBeInTheDocument();
    expect(screen.getByText("Medium (1-2s)")).toBeInTheDocument();
    expect(screen.getByText("Slow (>2s)")).toBeInTheDocument();
  });
});
