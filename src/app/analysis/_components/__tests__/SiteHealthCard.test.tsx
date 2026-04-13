import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { SiteHealthCard } from "../molecules/SiteHealthCard";

const analysis = {
  ssl_certificate: true,
  sitemap_found: true,
  robots_txt_found: false,
} as never;

const pages = [
  { mobile_friendly: true, has_structured_data: true },
  { mobile_friendly: true, has_structured_data: false },
  { mobile_friendly: false, has_structured_data: false },
] as never[];

describe("SiteHealthCard", () => {
  it("renders site health heading", () => {
    render(<SiteHealthCard analysis={analysis} pages={pages} />);
    expect(screen.getByText("Site Health")).toBeInTheDocument();
  });

  it("shows health indicators for SSL, Sitemap, robots.txt", () => {
    render(<SiteHealthCard analysis={analysis} pages={pages} />);
    expect(screen.getByText("SSL")).toBeInTheDocument();
    expect(screen.getByText("Sitemap")).toBeInTheDocument();
    expect(screen.getByText("robots.txt")).toBeInTheDocument();
  });

  it("shows mobile friendly count", () => {
    render(<SiteHealthCard analysis={analysis} pages={pages} />);
    expect(screen.getByText("Mobile Friendly")).toBeInTheDocument();
    expect(screen.getByText("2/3")).toBeInTheDocument();
  });

  it("shows structured data count", () => {
    render(<SiteHealthCard analysis={analysis} pages={pages} />);
    expect(screen.getByText("Structured Data")).toBeInTheDocument();
    expect(screen.getByText("1/3")).toBeInTheDocument();
  });
});
