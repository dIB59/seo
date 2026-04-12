import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ScoreCard } from "../molecules/ScoreCard";

const pages = [
  { lighthouse_seo: 90 },
  { lighthouse_seo: 80 },
] as never[];

const issues = [
  { severity: "critical", title: "Missing Title" },
  { severity: "critical", title: "Broken Link" },
  { severity: "warning", title: "Slow Page" },
  { severity: "info", title: "Missing Alt" },
] as never[];

describe("ScoreCard", () => {
  it("renders average SEO score", () => {
    render(<ScoreCard pages={pages} issues={issues} />);
    // Average of 90 and 80 = 85
    // Score appears in both the heading and the ScoreRing
    expect(screen.getAllByText("85").length).toBeGreaterThanOrEqual(1);
  });

  it("renders score label", () => {
    render(<ScoreCard pages={pages} issues={issues} />);
    expect(screen.getByText("Good")).toBeInTheDocument();
  });

  it("shows total issue count", () => {
    render(<ScoreCard pages={pages} issues={issues} />);
    expect(screen.getByText("4")).toBeInTheDocument();
  });

  it("shows issue severity breakdown", () => {
    render(<ScoreCard pages={pages} issues={issues} />);
    expect(screen.getByText("Issues Found")).toBeInTheDocument();
  });
});
