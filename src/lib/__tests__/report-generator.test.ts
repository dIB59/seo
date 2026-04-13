import { describe, it, expect } from "vitest";
import { generateReport, generateCSV } from "../report-generator";

const mockResult = {
  analysis: {
    id: "job-1",
    url: "https://example.com",
    status: "completed",
    progress: 100,
    total_pages: 2,
    analyzed_pages: 2,
    started_at: "2026-01-01T00:00:00Z",
    completed_at: "2026-01-01T00:01:00Z",
    sitemap_found: true,
    robots_txt_found: false,
    ssl_certificate: true,
    created_at: "2026-01-01T00:00:00Z",
  },
  summary: {
    analysis_id: "job-1",
    seo_score: 75,
    avg_load_time: 1.234,
    total_words: 5000,
    total_issues: 3,
  },
  pages: [
    {
      analysis_id: "job-1",
      url: "https://example.com/",
      title: "Home Page",
      meta_description: "Welcome",
      meta_keywords: null,
      canonical_url: null,
      word_count: 3000,
      image_count: 5,
      images_without_alt: 2,
      internal_links: 10,
      external_links: 3,
      load_time: 0.85,
      status_code: 200,
      content_size: 50000,
      mobile_friendly: true,
      has_structured_data: true,
      lighthouse_performance: null,
      lighthouse_accessibility: null,
      lighthouse_best_practices: null,
      lighthouse_seo: 92,
      lighthouse_seo_audits: null,
      lighthouse_performance_metrics: null,
      images: [],
      detailed_links: [],
      headings: [
        { tag: "h1", text: "Main" },
        { tag: "h2", text: "Sub" },
      ],
      extracted_data: {},
    },
    {
      analysis_id: "job-1",
      url: "https://example.com/about",
      title: null,
      meta_description: null,
      meta_keywords: null,
      canonical_url: null,
      word_count: 2000,
      image_count: 0,
      images_without_alt: 0,
      internal_links: 5,
      external_links: 1,
      load_time: 2.5,
      status_code: 200,
      content_size: 30000,
      mobile_friendly: false,
      has_structured_data: false,
      lighthouse_performance: null,
      lighthouse_accessibility: null,
      lighthouse_best_practices: null,
      lighthouse_seo: null,
      lighthouse_seo_audits: null,
      lighthouse_performance_metrics: null,
      images: [],
      detailed_links: [],
      headings: [],
      extracted_data: {},
    },
  ],
  issues: [
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
      page_url: "https://example.com/about",
      severity: "warning" as const,
      title: "Missing Meta Description",
      description: "No meta description found",
      element: null,
      recommendation: "Add a meta description",
      line_number: null,
    },
    {
      page_id: "p1",
      page_url: "https://example.com/",
      severity: "info" as const,
      title: "Images Without Alt",
      description: "2 images lack alt text",
      element: null,
      recommendation: "Add alt text to images",
      line_number: null,
    },
  ],
} as never;

describe("generateReport", () => {
  it("includes the site URL", () => {
    const report = generateReport(mockResult);
    expect(report).toContain("https://example.com");
  });

  it("includes the SEO score and label", () => {
    const report = generateReport(mockResult);
    expect(report).toContain("75/100");
    expect(report).toContain("Fair");
  });

  it("includes site health indicators", () => {
    const report = generateReport(mockResult);
    expect(report).toContain("SSL Certificate: Valid");
    expect(report).toContain("Sitemap: Found");
    expect(report).toContain("robots.txt: Not Found");
  });

  it("lists critical issues", () => {
    const report = generateReport(mockResult);
    expect(report).toContain("CRITICAL ISSUES (1)");
    expect(report).toContain("Missing Title");
  });

  it("lists warnings", () => {
    const report = generateReport(mockResult);
    expect(report).toContain("WARNINGS (1)");
    expect(report).toContain("Missing Meta Description");
  });

  it("includes page-by-page analysis", () => {
    const report = generateReport(mockResult);
    expect(report).toContain("Home Page");
    expect(report).toContain("https://example.com/about");
    expect(report).toContain("Missing"); // title missing for /about
  });

  it("shows heading counts", () => {
    const report = generateReport(mockResult);
    expect(report).toContain("H1(1)");
    expect(report).toContain("H2(1)");
  });
});

describe("generateCSV", () => {
  it("starts with a header row", () => {
    const csv = generateCSV(mockResult);
    const firstLine = csv.split("\n")[0];
    expect(firstLine).toContain("URL");
    expect(firstLine).toContain("Load Time");
    expect(firstLine).toContain("Word Count");
    expect(firstLine).toContain("Issues Found");
  });

  it("includes one row per page", () => {
    const csv = generateCSV(mockResult);
    const lines = csv.split("\n");
    expect(lines).toHaveLength(3); // header + 2 pages
  });

  it("escapes URLs with quotes", () => {
    const csv = generateCSV(mockResult);
    expect(csv).toContain('"https://example.com/"');
  });

  it("counts issues per page", () => {
    const csv = generateCSV(mockResult);
    const lines = csv.split("\n");
    // /about has 2 issues (critical + warning)
    const aboutLine = lines.find((l) => l.includes("about"));
    expect(aboutLine).toContain(",2");
    // / has 1 issue (info)
    const homeLine = lines.find((l) => l.includes("example.com/\""));
    expect(homeLine).toContain(",1");
  });

  it("formats load time to 2 decimal places", () => {
    const csv = generateCSV(mockResult);
    expect(csv).toContain("0.85");
    expect(csv).toContain("2.50");
  });
});
