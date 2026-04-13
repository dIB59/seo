import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import {
  formatUrlPath,
  SeoScore,
  LoadTime,
  WordsCell,
  HeadingCounts,
  LinksCell,
  ImageCount,
  PageInfo,
} from "../atoms/PageRowAtoms";

describe("formatUrlPath", () => {
  it("strips protocol and host", () => {
    expect(formatUrlPath("https://example.com/about")).toBe("/about");
    expect(formatUrlPath("http://example.com/page?q=1")).toBe("/page?q=1");
  });
  it("returns / for root URL", () => {
    expect(formatUrlPath("https://example.com")).toBe("/");
    expect(formatUrlPath("https://example.com/")).toBe("/");
  });
});

describe("SeoScore", () => {
  it("renders — for null score", () => {
    render(<SeoScore score={null} />);
    expect(screen.getByText("—")).toBeInTheDocument();
  });
  it("renders score with 3 significant digits", () => {
    render(<SeoScore score={85} />);
    expect(screen.getByText("85.0")).toBeInTheDocument();
  });
});

describe("LoadTime", () => {
  it("renders load time for healthy pages", () => {
    render(<LoadTime loadTime={1.23} isBroken={false} />);
    expect(screen.getByText("1.23s")).toBeInTheDocument();
  });
  it("renders with precision for broken pages", () => {
    render(<LoadTime loadTime={2.5} isBroken={true} />);
    expect(screen.getByText("2.5s")).toBeInTheDocument();
  });
});

describe("WordsCell", () => {
  it("renders word count with locale formatting", () => {
    render(<WordsCell count={1500} isBroken={false} />);
    expect(screen.getByText("1,500")).toBeInTheDocument();
  });
  it("renders — for broken pages", () => {
    render(<WordsCell count={0} isBroken={true} />);
    expect(screen.getByText("—")).toBeInTheDocument();
  });
});

describe("HeadingCounts", () => {
  it("renders h1/h2/h3 counts", () => {
    render(<HeadingCounts h1={1} h2={3} h3={5} isBroken={false} />);
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
  });
  it("renders — for broken pages", () => {
    render(<HeadingCounts h1={0} h2={0} h3={0} isBroken={true} />);
    expect(screen.getByText("—")).toBeInTheDocument();
  });
});

describe("LinksCell", () => {
  it("renders internal and external counts", () => {
    render(<LinksCell internal={10} external={3} isBroken={false} />);
    expect(screen.getByText("10")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
  });
  it("renders — for broken pages", () => {
    render(<LinksCell internal={0} external={0} isBroken={true} />);
    expect(screen.getByText("—")).toBeInTheDocument();
  });
});

describe("ImageCount", () => {
  it("renders image count", () => {
    render(<ImageCount count={5} withoutAlt={0} isBroken={false} />);
    expect(screen.getByText("5")).toBeInTheDocument();
  });
  it("shows alt warning badge when images missing alt", () => {
    render(<ImageCount count={5} withoutAlt={2} isBroken={false} />);
    expect(screen.getByText("2")).toBeInTheDocument();
  });
});

describe("PageInfo", () => {
  it("renders URL path and title", () => {
    render(
      <PageInfo url="https://example.com/about" title="About Us" isBroken={false} />,
    );
    expect(screen.getByText("/about")).toBeInTheDocument();
    expect(screen.getByText("About Us")).toBeInTheDocument();
  });
  it("shows 'No title' when title is null", () => {
    render(
      <PageInfo url="https://example.com/" title={null} isBroken={false} />,
    );
    expect(screen.getByText("No title")).toBeInTheDocument();
  });
  it("shows status code badge for broken pages", () => {
    render(
      <PageInfo url="https://example.com/broken" title="Broken" isBroken={true} statusCode={404} />,
    );
    expect(screen.getByText("404")).toBeInTheDocument();
  });
});
