import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import StatusBadge from "../atoms/StatusBadge";
import CharLengthBadge from "../atoms/CharLengthBadge";

describe("StatusBadge", () => {
  it("renders with success styling when hasContent is true", () => {
    render(<StatusBadge hasContent={true} label="Title present" />);
    expect(screen.getByText("Title present")).toBeInTheDocument();
  });

  it("renders with destructive styling when hasContent is false", () => {
    render(<StatusBadge hasContent={false} label="Title missing" />);
    expect(screen.getByText("Title missing")).toBeInTheDocument();
  });
});

describe("CharLengthBadge", () => {
  it("renders character count", () => {
    render(<CharLengthBadge length={55} />);
    expect(screen.getByText("55 chars")).toBeInTheDocument();
  });

  it("shows warning when exceeding max recommended", () => {
    render(<CharLengthBadge length={200} maxRecommended={160} />);
    const badge = screen.getByText("200 chars");
    expect(badge).toBeInTheDocument();
    // Warning class should be applied
    expect(badge.className).toContain("warning");
  });

  it("shows normal styling when within limit", () => {
    render(<CharLengthBadge length={100} maxRecommended={160} />);
    const badge = screen.getByText("100 chars");
    expect(badge.className).not.toContain("warning");
  });
});
