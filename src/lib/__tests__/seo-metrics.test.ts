import { describe, it, expect } from "vitest";
import {
  getScoreColor,
  getScoreBgColor,
  getScoreLabel,
  getLoadTimeColor,
} from "../seo-metrics";

describe("getScoreColor", () => {
  it("returns success for scores >= 80", () => {
    expect(getScoreColor(80)).toBe("text-success");
    expect(getScoreColor(100)).toBe("text-success");
  });
  it("returns warning for scores 50-79", () => {
    expect(getScoreColor(50)).toBe("text-warning");
    expect(getScoreColor(79)).toBe("text-warning");
  });
  it("returns destructive for scores < 50", () => {
    expect(getScoreColor(49)).toBe("text-destructive");
    expect(getScoreColor(0)).toBe("text-destructive");
  });
});

describe("getScoreBgColor", () => {
  it("returns bg-success for scores >= 80", () => {
    expect(getScoreBgColor(90)).toBe("bg-success");
  });
  it("returns bg-warning for scores 50-79", () => {
    expect(getScoreBgColor(65)).toBe("bg-warning");
  });
  it("returns bg-destructive for scores < 50", () => {
    expect(getScoreBgColor(30)).toBe("bg-destructive");
  });
});

describe("getScoreLabel", () => {
  it("returns Excellent for >= 90", () => expect(getScoreLabel(90)).toBe("Excellent"));
  it("returns Good for >= 80", () => expect(getScoreLabel(80)).toBe("Good"));
  it("returns Fair for >= 60", () => expect(getScoreLabel(60)).toBe("Fair"));
  it("returns Poor for >= 40", () => expect(getScoreLabel(40)).toBe("Poor"));
  it("returns Critical for < 40", () => expect(getScoreLabel(39)).toBe("Critical"));
});

describe("getLoadTimeColor", () => {
  it("returns success for < 1s", () => {
    expect(getLoadTimeColor(0.5)).toBe("text-success");
  });
  it("returns warning for 1-2s", () => {
    expect(getLoadTimeColor(1.5)).toBe("text-warning");
  });
  it("returns destructive for >= 2s", () => {
    expect(getLoadTimeColor(3)).toBe("text-destructive");
  });
});
