import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";

// Mock the hook so we don't need a real Tauri backend
vi.mock("@/src/hooks/use-extractor-tags", () => ({
  useExtractorTags: () => ({
    tags: [],
    extractorTags: [
      {
        name: "tag:og_image",
        label: "OG Image",
        description: "Custom extractor — CSS: meta[property='og:image'] @content",
        dataType: "text",
        source: { kind: "extractor", extractor_id: "ext-1", extractor_name: "OG Image" },
        scopes: ["checkField", "templateText"],
        example: "https://img.jpg",
      },
    ],
    builtinTags: [
      {
        name: "url",
        label: "Site URL",
        description: "The base URL of the site being audited.",
        dataType: "text",
        source: { kind: "builtin" },
        scopes: ["templateText", "aiPrompt"],
        example: "https://example.com",
      },
      {
        name: "score",
        label: "SEO Score",
        description: "Overall SEO score (0–100).",
        dataType: "number",
        source: { kind: "builtin" },
        scopes: ["templateText", "templateCondition"],
        example: "87",
      },
    ],
    isLoading: false,
  }),
}));

import { TagsSettings } from "../TagsSettings";

describe("TagsSettings", () => {
  it("renders the extractor tags section", () => {
    render(<TagsSettings />);
    expect(screen.getByText("Your Extractor Tags")).toBeInTheDocument();
    expect(screen.getByText("tag:og_image")).toBeInTheDocument();
    expect(screen.getAllByText("OG Image").length).toBeGreaterThanOrEqual(1);
  });

  it("renders the built-in tags section", () => {
    render(<TagsSettings />);
    expect(screen.getByText("Built-in Tags")).toBeInTheDocument();
    expect(screen.getByText("url")).toBeInTheDocument();
    expect(screen.getByText("Site URL")).toBeInTheDocument();
  });

  it("shows data type badges", () => {
    render(<TagsSettings />);
    expect(screen.getAllByText("text").length).toBeGreaterThan(0);
    expect(screen.getByText("number")).toBeInTheDocument();
  });

  it("shows scope badges for tags", () => {
    render(<TagsSettings />);
    // The extractor tag has checkField and templateText scopes
    expect(screen.getAllByText("Check").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Template").length).toBeGreaterThan(0);
  });

  it("displays example values", () => {
    render(<TagsSettings />);
    expect(screen.getByText("https://img.jpg")).toBeInTheDocument();
    expect(screen.getByText("87")).toBeInTheDocument();
  });
});
