import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { QuickStatsCard } from "../molecules/QuickStat";

const summary = {
  analysis_id: "j1",
  seo_score: 80,
  avg_load_time: 1.5,
  total_words: 12000,
  total_issues: 5,
} as never;

const pages = [
  { image_count: 3, internal_links: 10, load_time: 0.5 },
  { image_count: 5, internal_links: 8, load_time: 1.5 },
  { image_count: 2, internal_links: 12, load_time: 2.0 },
] as never[];

describe("QuickStatsCard", () => {
  it("renders Quick Stats heading", () => {
    render(<QuickStatsCard summary={summary} pages={pages} />);
    expect(screen.getByText("Quick Stats")).toBeInTheDocument();
  });

  it("shows total images across all pages", () => {
    render(<QuickStatsCard summary={summary} pages={pages} />);
    // 3 + 5 + 2 = 10
    expect(screen.getByText("10")).toBeInTheDocument();
  });

  it("shows total internal links", () => {
    render(<QuickStatsCard summary={summary} pages={pages} />);
    // 10 + 8 + 12 = 30
    expect(screen.getByText("30")).toBeInTheDocument();
  });

  it("shows total words from summary", () => {
    render(<QuickStatsCard summary={summary} pages={pages} />);
    expect(screen.getByText("12000")).toBeInTheDocument();
  });

  it("shows average load time", () => {
    render(<QuickStatsCard summary={summary} pages={pages} />);
    // (0.5 + 1.5 + 2.0) / 3 = 1.33
    expect(screen.getByText("1.33")).toBeInTheDocument();
  });

  it("handles empty pages without NaN", () => {
    render(<QuickStatsCard summary={summary} pages={[]} />);
    expect(screen.getByText("0.00")).toBeInTheDocument(); // avg load time = 0
    // Multiple "0" values (images=0, links=0) — just verify no NaN
    const allText = document.body.textContent ?? "";
    expect(allText).not.toContain("NaN");
  });
});
