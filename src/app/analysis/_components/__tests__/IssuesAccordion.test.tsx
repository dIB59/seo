import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";

// Mock Tauri shell plugin
vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn(),
}));

import { IssuesAccordion } from "../organisms/IssuesAccordion";

const issues = [
  {
    page_id: "p1",
    page_url: "https://example.com/about",
    severity: "critical" as const,
    title: "Missing Title",
    description: "Page has no title tag",
    element: null,
    recommendation: "Add a title tag",
    line_number: null,
  },
  {
    page_id: "p2",
    page_url: "https://example.com/contact",
    severity: "critical" as const,
    title: "Missing Title",
    description: "Page has no title tag",
    element: null,
    recommendation: "Add a title tag",
    line_number: null,
  },
  {
    page_id: "p3",
    page_url: "https://example.com/blog",
    severity: "warning" as const,
    title: "Slow Load",
    description: "Page takes >3s to load",
    element: null,
    recommendation: "Optimize images",
    line_number: null,
  },
];

describe("IssuesAccordion", () => {
  it("renders empty state when no issues", () => {
    render(<IssuesAccordion issues={[]} />);
    expect(screen.getByText("No issues found. Great job!")).toBeInTheDocument();
  });

  it("groups issues by title", () => {
    render(<IssuesAccordion issues={issues} />);
    // "Missing Title" appears once as a group header (with count)
    expect(screen.getByText("Missing Title")).toBeInTheDocument();
    expect(screen.getByText("2 pages affected")).toBeInTheDocument();
    // "Slow Load" appears as its own group
    expect(screen.getByText("Slow Load")).toBeInTheDocument();
    expect(screen.getByText("1 page affected")).toBeInTheDocument();
  });

  it("shows View Details badge on each group", () => {
    render(<IssuesAccordion issues={issues} />);
    const badges = screen.getAllByText("View Details");
    expect(badges).toHaveLength(2); // one per group
  });
});
