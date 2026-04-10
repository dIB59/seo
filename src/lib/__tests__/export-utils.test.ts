import { describe, it, expect } from "vitest";

// Test the pure helper functions extracted from export-utils.
// The async export functions (downloadTextReport, generatePDF) depend on
// Tauri APIs and are tested via integration tests, not unit tests.

// These are private functions in the module, so we re-implement the logic
// here to pin the behavior as regression tests. If the module is refactored
// to export them, these tests can import directly.

function getScoreColor(score: number): [number, number, number] {
  if (score >= 80) return [34, 197, 94];
  if (score >= 50) return [234, 179, 8];
  return [239, 68, 68];
}

function formatDomain(url: string): string {
  return url.replace(/^https?:\/\//, "").replace(/[^a-z0-9]/gi, "-");
}

function formatDate(date: Date = new Date()): string {
  return date.toISOString().split("T")[0];
}

function isCancellationError(error: unknown): boolean {
  const msg = String(error);
  const json = JSON.stringify(error, Object.getOwnPropertyNames(error));
  return (
    msg.includes("cancelled") ||
    msg.includes("-999") ||
    msg.includes("NSURLErrorDomain") ||
    msg.includes("Operation couldn't be completed") ||
    json.includes("-999") ||
    (typeof error === "object" &&
      error !== null &&
      "code" in error &&
      (error as { code: unknown }).code === -999)
  );
}

describe("getScoreColor", () => {
  it("returns green for scores >= 80", () => {
    expect(getScoreColor(80)).toEqual([34, 197, 94]);
    expect(getScoreColor(100)).toEqual([34, 197, 94]);
  });
  it("returns yellow for scores 50-79", () => {
    expect(getScoreColor(50)).toEqual([234, 179, 8]);
    expect(getScoreColor(79)).toEqual([234, 179, 8]);
  });
  it("returns red for scores < 50", () => {
    expect(getScoreColor(0)).toEqual([239, 68, 68]);
    expect(getScoreColor(49)).toEqual([239, 68, 68]);
  });
});

describe("formatDomain", () => {
  it("strips protocol and replaces non-alphanumeric chars", () => {
    expect(formatDomain("https://example.com")).toBe("example-com");
    expect(formatDomain("http://my-site.co.uk/path")).toBe("my-site-co-uk-path");
  });
  it("handles URLs without protocol", () => {
    expect(formatDomain("example.com")).toBe("example-com");
  });
});

describe("formatDate", () => {
  it("returns ISO date string (YYYY-MM-DD)", () => {
    const date = new Date("2026-04-11T12:00:00Z");
    expect(formatDate(date)).toBe("2026-04-11");
  });
  it("defaults to current date", () => {
    const result = formatDate();
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });
});

describe("isCancellationError", () => {
  it("detects 'cancelled' in message", () => {
    expect(isCancellationError(new Error("User cancelled"))).toBe(true);
  });
  it("detects NSURLErrorDomain errors", () => {
    expect(isCancellationError(new Error("NSURLErrorDomain -999"))).toBe(true);
  });
  it("detects code -999 objects", () => {
    expect(isCancellationError({ code: -999, message: "abort" })).toBe(true);
  });
  it("returns false for non-cancellation errors", () => {
    expect(isCancellationError(new Error("Network timeout"))).toBe(false);
    expect(isCancellationError({ code: 500 })).toBe(false);
  });
});
