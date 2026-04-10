import { describe, it, expect } from "vitest";
import { urlSchema, createSchema, normalizeUrl } from "../schema";

describe("urlSchema", () => {
  it("accepts valid URLs with protocol", () => {
    expect(urlSchema.safeParse("https://example.com").success).toBe(true);
    expect(urlSchema.safeParse("http://example.com").success).toBe(true);
    expect(urlSchema.safeParse("https://sub.example.co.uk/path").success).toBe(true);
  });

  it("accepts URLs without protocol (auto-prepends https)", () => {
    expect(urlSchema.safeParse("example.com").success).toBe(true);
    expect(urlSchema.safeParse("blog.example.com").success).toBe(true);
  });

  it("rejects empty strings", () => {
    expect(urlSchema.safeParse("").success).toBe(false);
    expect(urlSchema.safeParse("   ").success).toBe(false);
  });

  it("rejects strings without a dot (no TLD)", () => {
    expect(urlSchema.safeParse("localhost").success).toBe(false);
    expect(urlSchema.safeParse("just-a-word").success).toBe(false);
  });

  it("rejects non-http protocols", () => {
    expect(urlSchema.safeParse("ftp://example.com").success).toBe(false);
    expect(urlSchema.safeParse("javascript:alert(1)").success).toBe(false);
  });
});

describe("createSchema", () => {
  it("enforces max_pages limit based on tier", () => {
    const schema = createSchema(10);
    const valid = schema.safeParse({
      url: "https://example.com",
      settings: {
        max_pages: 10,
        include_subdomains: false,
        check_images: true,
        mobile_analysis: false,
        lighthouse_analysis: false,
        delay_between_requests: 50,
      },
    });
    expect(valid.success).toBe(true);

    const tooMany = schema.safeParse({
      url: "https://example.com",
      settings: {
        max_pages: 11,
        include_subdomains: false,
        check_images: true,
        mobile_analysis: false,
        lighthouse_analysis: false,
        delay_between_requests: 50,
      },
    });
    expect(tooMany.success).toBe(false);
  });

  it("rejects delay_between_requests > 5000", () => {
    const schema = createSchema(100);
    const result = schema.safeParse({
      url: "https://example.com",
      settings: {
        max_pages: 10,
        include_subdomains: false,
        check_images: true,
        mobile_analysis: false,
        lighthouse_analysis: false,
        delay_between_requests: 6000,
      },
    });
    expect(result.success).toBe(false);
  });
});

describe("normalizeUrl", () => {
  it("prepends https:// when missing", () => {
    expect(normalizeUrl("example.com")).toBe("https://example.com/");
  });

  it("preserves existing http://", () => {
    expect(normalizeUrl("http://example.com")).toBe("http://example.com/");
  });

  it("preserves existing https://", () => {
    expect(normalizeUrl("https://example.com/path")).toBe("https://example.com/path");
  });

  it("trims whitespace", () => {
    expect(normalizeUrl("  example.com  ")).toBe("https://example.com/");
  });

  it("returns input for unparseable strings", () => {
    expect(normalizeUrl("not a url at all")).toBe("not a url at all");
  });
});
